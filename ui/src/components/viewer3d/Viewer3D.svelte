<script>
  import { onMount, onDestroy } from "svelte";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { _ } from "svelte-i18n";

  let { visible = true } = $props();

  let container = $state(null);
  let canvas = $state(null);
  let THREE = $state(null);
  let loading = $state(true);
  let loadError = $state(false);

  // Three.js objects
  let renderer, scene, camera;
  let animationId;
  let kozijnGroup = null;

  // Camera orbit state
  let isDragging = false;
  let previousMouse = { x: 0, y: 0 };
  let spherical = { theta: Math.PI / 4, phi: Math.PI / 3, radius: 2500 };
  let target = { x: 0, y: 0, z: 0 };

  // Material colors
  const MATERIAL_COLORS = {
    "wood(meranti)": 0xC4956A,
    "wood(accoya)": 0xD4C5A9,
    "wood(vuren)": 0xE8D5A3,
    "wood(eiken)": 0xA07040,
    aluminum: 0xB0B0B0,
    pvc: 0xF0F0F0,
    "wood_aluminum": 0xC4956A,
  };

  const GLASS_COLOR = 0x88CCDD;
  const PANEL_COLOR = 0xD0C8B8;
  const DOOR_COLOR = 0xA08060;

  // Operable sash types (mirror PanelType::is_operable in ofs-core).
  const OPERABLE_TYPES = new Set([
    "turn_tilt", "turn", "tilt", "sliding", "door",
    "top_hung", "bottom_hung", "lift_slide", "pivot",
  ]);

  // Panel-filling tint per FillingType (serde snake_case).
  const FILLING_COLORS = {
    sandwich: 0xD0C8B8,
    solid: 0xB8A888,
    door_panel: 0xA08060,
    ventilation: 0x9AA0A8,
    blind: 0xC0C0C0,
  };

  function getMaterialKey(material) {
    if (!material) return "wood(meranti)";
    if (typeof material === "string") return material.toLowerCase();
    // Rust enum serialized as { "wood": "meranti" } or "aluminum"
    if (typeof material === "object") {
      const key = Object.keys(material)[0];
      const val = material[key];
      return `${key}(${val})`.toLowerCase();
    }
    return "wood(meranti)";
  }

  function getFrameColor(kozijn) {
    const key = getMaterialKey(kozijn?.frame?.material);
    return MATERIAL_COLORS[key] || MATERIAL_COLORS["wood(meranti)"];
  }

  async function loadThreeJS() {
    try {
      const timeout = new Promise((_, reject) =>
        setTimeout(() => reject(new Error("Three.js load timeout (10s)")), 10000)
      );
      THREE = await Promise.race([import("three"), timeout]);
      loading = false;
      return true;
    } catch (e) {
      console.error("Failed to load Three.js:", e);
      loadError = true;
      loading = false;
      return false;
    }
  }

  function initScene() {
    if (!THREE || !canvas || !container) return;

    // WebGL detection
    const testCtx = canvas.getContext('webgl2') || canvas.getContext('webgl');
    if (!testCtx) {
      console.error("WebGL not available in this webview");
      loadError = true;
      return;
    }

    // Renderer
    renderer = new THREE.WebGLRenderer({
      canvas,
      antialias: true,
      alpha: false,
    });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.setClearColor(0x1a1a2e, 1);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    renderer.toneMapping = THREE.ACESFilmicToneMapping;
    renderer.toneMappingExposure = 1.0;

    // Scene
    scene = new THREE.Scene();

    // Camera
    camera = new THREE.PerspectiveCamera(45, 1, 1, 50000);
    updateCameraPosition();

    // Lighting
    const ambient = new THREE.AmbientLight(0xffffff, 0.5);
    scene.add(ambient);

    const dirLight = new THREE.DirectionalLight(0xffffff, 0.8);
    dirLight.position.set(1500, 2000, 1000);
    dirLight.castShadow = true;
    dirLight.shadow.mapSize.width = 1024;
    dirLight.shadow.mapSize.height = 1024;
    scene.add(dirLight);

    const fillLight = new THREE.DirectionalLight(0x8899bb, 0.3);
    fillLight.position.set(-1000, 500, -500);
    scene.add(fillLight);

    // Hemisphere light for sky/ground ambient
    const hemiLight = new THREE.HemisphereLight(0x87ceeb, 0x362e1c, 0.3);
    scene.add(hemiLight);

    // Grid helper
    const gridHelper = new THREE.GridHelper(4000, 40, 0x333355, 0x222244);
    gridHelper.position.y = -0.5;
    scene.add(gridHelper);

    // Initial sizing
    handleResize();

    // Render loop
    function animate() {
      animationId = requestAnimationFrame(animate);
      renderer.render(scene, camera);
    }
    animate();
  }

  function updateCameraPosition() {
    if (!camera) return;
    const { theta, phi, radius } = spherical;
    camera.position.set(
      target.x + radius * Math.sin(phi) * Math.cos(theta),
      target.y + radius * Math.cos(phi),
      target.z + radius * Math.sin(phi) * Math.sin(theta)
    );
    camera.lookAt(target.x, target.y, target.z);
  }

  function handleResize() {
    if (!container || !renderer || !camera) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w === 0 || h === 0) return;
    renderer.setSize(w, h);
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
  }

  // --- Mouse orbit controls ---

  function onMouseDown(e) {
    if (e.button === 0) {
      isDragging = true;
      previousMouse = { x: e.clientX, y: e.clientY };
      e.preventDefault();
    }
  }

  function onMouseMove(e) {
    if (!isDragging) return;
    const dx = e.clientX - previousMouse.x;
    const dy = e.clientY - previousMouse.y;
    previousMouse = { x: e.clientX, y: e.clientY };

    spherical.theta -= dx * 0.005;
    spherical.phi -= dy * 0.005;
    // Clamp phi to avoid flipping
    spherical.phi = Math.max(0.1, Math.min(Math.PI - 0.1, spherical.phi));

    updateCameraPosition();
  }

  function onMouseUp() {
    isDragging = false;
  }

  function onWheel(e) {
    e.preventDefault();
    spherical.radius *= e.deltaY > 0 ? 1.1 : 0.9;
    spherical.radius = Math.max(200, Math.min(15000, spherical.radius));
    updateCameraPosition();
  }

  // --- 3D Kozijn Builder ---

  function build3DKozijn(scene, kozijn, geometry) {
    // Remove previous kozijn group
    if (kozijnGroup) {
      scene.remove(kozijnGroup);
      kozijnGroup.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
          if (Array.isArray(child.material)) {
            child.material.forEach((m) => m.dispose());
          } else {
            child.material.dispose();
          }
        }
      });
    }

    if (!kozijn || !geometry) return;

    kozijnGroup = new THREE.Group();
    const frameColor = getFrameColor(kozijn);
    const frameDepth = kozijn.frame?.frameDepth || 114;

    // Frame material — PBR properties depend on material type
    const matKey = getMaterialKey(kozijn?.frame?.material);
    const isAluminum = matKey === "aluminum";
    const frameMat = new THREE.MeshStandardMaterial({
      color: frameColor,
      roughness: isAluminum ? 0.3 : 0.7,
      metalness: isAluminum ? 0.8 : 0.0,
    });

    // Glass material — realistic PBR glass
    const glassMat = new THREE.MeshPhysicalMaterial({
      color: GLASS_COLOR,
      transparent: true,
      opacity: 0.3,
      roughness: 0.05,
      metalness: 0.0,
      transmission: 0.9,
      ior: 1.5,
      thickness: 0.5,
      side: THREE.DoubleSide,
    });

    // Door material
    const doorMat = new THREE.MeshStandardMaterial({
      color: DOOR_COLOR,
      roughness: 0.6,
      metalness: 0.0,
    });

    // Divider material (same as frame)
    const dividerMat = frameMat.clone();

    // Sash frame material (matches the frame; slightly different sheen)
    const sashMat = new THREE.MeshStandardMaterial({
      color: frameColor,
      roughness: isAluminum ? 0.3 : 0.65,
      metalness: isAluminum ? 0.8 : 0.0,
    });

    // Glazing-bead material (glaslat) — matches the frame colour
    const beadMat = new THREE.MeshStandardMaterial({
      color: frameColor,
      roughness: isAluminum ? 0.35 : 0.6,
      metalness: isAluminum ? 0.7 : 0.0,
    });

    // Helper: create a box mesh from a 2D rect, extruded into Z
    function makeBox(rect, depth, material, zOffset = 0) {
      const geo = new THREE.BoxGeometry(rect.width, rect.height, depth);
      const mesh = new THREE.Mesh(geo, material);
      // Position: rect x,y are top-left in 2D, convert to center for Three.js
      mesh.position.set(
        rect.x + rect.width / 2,
        -(rect.y + rect.height / 2), // flip Y (2D y-down to 3D y-up)
        zOffset
      );
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      return mesh;
    }

    // Helper: shrink a rect inwards by `by` mm on every side
    function insetRectBy(rect, by) {
      return {
        x: rect.x + by,
        y: rect.y + by,
        width: Math.max(1, rect.width - 2 * by),
        height: Math.max(1, rect.height - 2 * by),
      };
    }

    // Helper: add a 4-member border (top/bottom/left/right) of `memberWidth`
    // around `rect`, extruded `depth` in Z at `zOffset`. Used for sash frames
    // and glazing beads.
    function addBorder(rect, memberWidth, depth, material, zOffset = 0) {
      const w = Math.min(memberWidth, rect.width / 2, rect.height / 2);
      const innerH = Math.max(1, rect.height - 2 * w);
      kozijnGroup.add(makeBox({ x: rect.x, y: rect.y, width: rect.width, height: w }, depth, material, zOffset));
      kozijnGroup.add(makeBox({ x: rect.x, y: rect.y + rect.height - w, width: rect.width, height: w }, depth, material, zOffset));
      kozijnGroup.add(makeBox({ x: rect.x, y: rect.y + w, width: w, height: innerH }, depth, material, zOffset));
      kozijnGroup.add(makeBox({ x: rect.x + rect.width - w, y: rect.y + w, width: w, height: innerH }, depth, material, zOffset));
    }

    // Frame members
    if (geometry.frameRects) {
      for (const rect of geometry.frameRects) {
        kozijnGroup.add(makeBox(rect, frameDepth, frameMat));
      }
    }

    // Cell contents: glass, sash frames, panel fillings, glazing beads
    // Glass sits in the sponning (rebate), offset toward the outside (-Z).
    const GLASS_CLEARANCE = 4; // mm clearance per side
    const glassThickness = kozijn.cells?.[0]?.glazing?.thicknessMm || 24;
    const glassZOffset = frameDepth * 0.3 - frameDepth / 2;

    if (geometry.cellRects) {
      for (const cellRect of geometry.cellRects) {
        const cell = kozijn.cells?.[cellRect.cellIndex];
        const panelType = cell?.panelType || "fixed_glass";
        const rect = cellRect.rect;
        const cellInset = insetRectBy(rect, GLASS_CLEARANCE);

        // Vakvulling (infill panel / ventilation grille)
        if (panelType === "panel" || panelType === "ventilation") {
          const filling = cell?.panelFilling;
          const thickness = filling?.thicknessMm || frameDepth * 0.6;
          const tint = FILLING_COLORS[filling?.fillingType] ||
            (panelType === "ventilation" ? FILLING_COLORS.ventilation : PANEL_COLOR);
          const fillMat = new THREE.MeshStandardMaterial({ color: tint, roughness: 0.8, metalness: 0.0 });
          kozijnGroup.add(makeBox(cellInset, thickness, fillMat, 0));

          // Ventilation grille — horizontal louver slats proud of the panel
          if (panelType === "ventilation") {
            const slatCount = Math.max(2, Math.min(8, Math.floor(cellInset.height / 90)));
            const slatMat = new THREE.MeshStandardMaterial({ color: 0x6B7178, roughness: 0.5, metalness: 0.3 });
            const gap = cellInset.height / (slatCount + 1);
            for (let s = 1; s <= slatCount; s++) {
              kozijnGroup.add(makeBox(
                { x: cellInset.x + 6, y: cellInset.y + gap * s - 4, width: Math.max(1, cellInset.width - 12), height: 8 },
                6, slatMat, thickness / 2 + 3
              ));
            }
          }
          continue;
        }

        // Glazed cell (fixed glass, operable sash, or door)
        const isOperable = OPERABLE_TYPES.has(panelType);
        let glassHostRect = cellInset;

        if (isOperable) {
          // Sash frame: a border of sash_width around the cell opening
          const sashWidth = cell?.sashWidth || 67;
          const sashDepth = frameDepth * 0.82;
          const sashZ = frameDepth * 0.06; // slightly proud on the inside
          addBorder(cellInset, sashWidth, sashDepth, sashMat, sashZ);
          glassHostRect = insetRectBy(cellInset, sashWidth);
        }

        if (panelType === "door") {
          // Door leaf — opaque infill inside the door sash frame
          kozijnGroup.add(makeBox(glassHostRect, frameDepth * 0.7, doorMat, 0));
        } else {
          // Glass pane
          const thisGlassThickness = cell?.glazing?.thicknessMm || glassThickness;
          kozijnGroup.add(makeBox(glassHostRect, thisGlassThickness, glassMat, glassZOffset));

          // Glaslat (glazing beads) — border around the glass, inside or outside
          const gl = cell?.glaslat;
          if (gl) {
            const beadWidth = gl.widthMm || 15;
            const beadDepth = gl.heightMm || 17;
            const inside = gl.position !== "buiten"; // default binnen (inside)
            const dir = inside ? 1 : -1;
            const glassFace = glassZOffset + dir * (thisGlassThickness / 2);
            const beadZ = glassFace + dir * (beadDepth / 2);
            addBorder(glassHostRect, beadWidth, beadDepth, beadMat, beadZ);
          }
        }
      }
    }

    // Vertical dividers
    if (geometry.vDividers) {
      for (const rect of geometry.vDividers) {
        kozijnGroup.add(makeBox(rect, frameDepth, dividerMat));
      }
    }

    // Horizontal dividers
    if (geometry.hDividers) {
      for (const rect of geometry.hDividers) {
        kozijnGroup.add(makeBox(rect, frameDepth, dividerMat));
      }
    }

    // Wall context — gray wall behind the kozijn with an opening cutout.
    // Centre it on the kozijn's actual bounding box (the members span y∈[-h,0],
    // so the centre is below the origin — use the measured centre, not 0).
    const preBbox = new THREE.Box3().setFromObject(kozijnGroup);
    const preSize = preBbox.getSize(new THREE.Vector3());
    const preCenter = preBbox.getCenter(new THREE.Vector3());
    const wallDepth = 300; // 300mm wall
    const wallSize = { x: preSize.x + 600, y: preSize.y + 400 };
    const wallGeo = new THREE.BoxGeometry(wallSize.x, wallSize.y, wallDepth);
    const wallMat = new THREE.MeshStandardMaterial({ color: 0xd0c8b8, roughness: 0.9, metalness: 0.0 });
    const wall = new THREE.Mesh(wallGeo, wallMat);
    wall.position.set(preCenter.x, preCenter.y, -wallDepth / 2 - frameDepth / 2);
    wall.receiveShadow = true;
    kozijnGroup.add(wall);

    // Opening cutout (simple: a dark box behind the glass, sized to the opening)
    const openingGeo = new THREE.BoxGeometry(preSize.x + 2, preSize.y + 2, wallDepth + 2);
    const openingMat = new THREE.MeshBasicMaterial({ color: 0x1a1a2e });
    const opening = new THREE.Mesh(openingGeo, openingMat);
    opening.position.set(preCenter.x, preCenter.y, -wallDepth / 2 - frameDepth / 2);
    kozijnGroup.add(opening);

    // Center the model
    const bbox = new THREE.Box3().setFromObject(kozijnGroup);
    const center = bbox.getCenter(new THREE.Vector3());
    kozijnGroup.position.sub(center);

    // Shift up so the bottom sits near the grid
    const size = bbox.getSize(new THREE.Vector3());
    kozijnGroup.position.y += size.y / 2;

    scene.add(kozijnGroup);

    // Adjust camera distance based on kozijn size
    const maxDim = Math.max(size.x, size.y, size.z);
    spherical.radius = maxDim * 1.8;
    target = { x: 0, y: size.y / 2, z: 0 };
    updateCameraPosition();
  }

  // --- Reactive rebuild ---
  // Use JSON serialization to detect deep changes (Svelte 5 $effect tracks references only)
  let prevGeomJson = $state("");

  $effect(() => {
    const k = $currentKozijn;
    const g = $currentGeometry;
    if (!scene || !k || !g) return;
    // Per-cell signature so type / sash / glaslat / filling / glass changes rebuild the model
    const cellSig = (k.cells || []).map((c) => [
      c.panelType,
      c.sashWidth || "",
      c.glaslat ? `${c.glaslat.position}${c.glaslat.widthMm}${c.glaslat.heightMm}` : "",
      c.panelFilling ? `${c.panelFilling.fillingType}${c.panelFilling.thicknessMm}` : "",
      c.glazing?.thicknessMm || "",
    ].join(":")).join("|");
    const newJson = JSON.stringify({ id: k.id, w: k.frame?.outerWidth, h: k.frame?.outerHeight, cells: k.cells?.length, shape: k.frame?.shape?.shapeType, sig: cellSig });
    if (newJson !== prevGeomJson) {
      prevGeomJson = newJson;
      build3DKozijn(scene, k, g);
    }
  });

  // --- Lifecycle ---

  let resizeObserver;

  let threeLoaded = $state(false);

  onMount(async () => {
    threeLoaded = await loadThreeJS();
  });

  // Initialize scene when visible becomes true AND Three.js is loaded
  $effect(() => {
    if (visible && threeLoaded && container && canvas && !renderer) {
      try {
        initScene();
        if ($currentKozijn && $currentGeometry) {
          build3DKozijn(scene, $currentKozijn, $currentGeometry);
        }
        resizeObserver = new ResizeObserver(() => handleResize());
        resizeObserver.observe(container);
      } catch (e) {
        console.error("3D scene init failed:", e);
        loadError = true;
      }
    }
  });

  onDestroy(() => {
    if (animationId) cancelAnimationFrame(animationId);
    if (resizeObserver) resizeObserver.disconnect();
    if (renderer) {
      renderer.dispose();
      renderer = null;
    }
    if (kozijnGroup) {
      kozijnGroup.traverse((child) => {
        if (child.geometry) child.geometry.dispose();
        if (child.material) {
          if (Array.isArray(child.material)) {
            child.material.forEach((m) => m.dispose());
          } else {
            child.material.dispose();
          }
        }
      });
    }
    scene = null;
    camera = null;
  });
</script>

{#if visible}
  <div
    class="viewer3d-container"
    bind:this={container}
    onmousedown={onMouseDown}
    onmousemove={onMouseMove}
    onmouseup={onMouseUp}
    onmouseleave={onMouseUp}
    onwheel={(e) => { e.preventDefault(); onWheel(e); }}
    role="img"
    aria-label="3D kozijn preview"
  >
    {#if loading}
      <div class="overlay">
        <div class="spinner"></div>
        <p>{$_('viewer3d.loading')}</p>
      </div>
    {:else if loadError}
      <div class="overlay">
        <p class="error">{$_('viewer3d.loadError')}</p>
      </div>
    {:else if !$currentKozijn}
      <div class="overlay">
        <p class="placeholder">{$_('viewer3d.selectKozijn')}</p>
      </div>
    {/if}

    <canvas bind:this={canvas}></canvas>
  </div>
{/if}

<style>
  .viewer3d-container {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 200px;
    background: var(--bg-app);
    overflow: hidden;
    cursor: default;
    user-select: none;
    border-radius: 4px;
  }

  .viewer3d-container:active {
    cursor: default;
  }

  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 1;
    pointer-events: none;
  }

  .overlay p {
    color: #8888aa;
    font-size: 14px;
    margin: 0;
  }

  .overlay .error {
    color: #ff6666;
  }

  .overlay .placeholder {
    color: #666688;
    font-size: 15px;
    font-style: italic;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid #333355;
    border-top-color: #6666aa;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-bottom: 12px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
