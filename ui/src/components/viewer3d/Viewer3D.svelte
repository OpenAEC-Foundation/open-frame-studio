<script>
  import { onMount, onDestroy } from "svelte";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { _ } from "svelte-i18n";

  let { visible = true } = $props();

  let container;
  let canvas;
  let THREE = null;
  let loading = true;
  let loadError = false;

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
      THREE = await import("three");
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

    // Frame material
    const frameMat = new THREE.MeshStandardMaterial({
      color: frameColor,
      roughness: 0.7,
      metalness: 0.0,
    });

    // Glass material
    const glassMat = new THREE.MeshPhysicalMaterial({
      color: GLASS_COLOR,
      transparent: true,
      opacity: 0.3,
      roughness: 0.1,
      metalness: 0.0,
      transmission: 0.6,
      thickness: 0.5,
      side: THREE.DoubleSide,
    });

    // Panel material (opaque for door/panel types)
    const panelMat = new THREE.MeshStandardMaterial({
      color: PANEL_COLOR,
      roughness: 0.8,
      metalness: 0.0,
    });

    // Door material
    const doorMat = new THREE.MeshStandardMaterial({
      color: DOOR_COLOR,
      roughness: 0.6,
      metalness: 0.0,
    });

    // Divider material (same as frame)
    const dividerMat = frameMat.clone();

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

    // Frame members
    if (geometry.frameRects) {
      for (const rect of geometry.frameRects) {
        kozijnGroup.add(makeBox(rect, frameDepth, frameMat));
      }
    }

    // Cell panels (glass, door, panel)
    // Glass sits in the sponning (rebate), offset toward the outside
    const GLASS_CLEARANCE = 4; // mm clearance per side
    const glassThickness = kozijn.cells?.[0]?.glazing?.thicknessMm || 24;
    // Sponning position: glass sits ~30% from the outside face
    const glassZOffset = frameDepth * 0.3 - frameDepth / 2;

    if (geometry.cellRects) {
      for (const cellRect of geometry.cellRects) {
        const cell = kozijn.cells?.[cellRect.cellIndex];
        const panelType = cell?.panelType || "fixed_glass";
        const rect = cellRect.rect;

        // Apply glass clearance — glass/panel is smaller than the cell opening
        const insetRect = {
          x: rect.x + GLASS_CLEARANCE,
          y: rect.y + GLASS_CLEARANCE,
          width: Math.max(1, rect.width - 2 * GLASS_CLEARANCE),
          height: Math.max(1, rect.height - 2 * GLASS_CLEARANCE),
        };

        if (panelType === "panel") {
          kozijnGroup.add(makeBox(insetRect, frameDepth * 0.6, panelMat, 0));
        } else if (panelType === "door") {
          kozijnGroup.add(makeBox(insetRect, frameDepth * 0.7, doorMat, 0));
        } else {
          // Glass types: fixed_glass, turn_tilt, turn, tilt, sliding
          const thisGlassThickness = cell?.glazing?.thicknessMm || glassThickness;
          kozijnGroup.add(makeBox(insetRect, thisGlassThickness, glassMat, glassZOffset));
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

  let prevKozijn = null;
  let prevGeometry = null;

  $effect(() => {
    if (scene && ($currentKozijn !== prevKozijn || $currentGeometry !== prevGeometry)) {
      prevKozijn = $currentKozijn;
      prevGeometry = $currentGeometry;
      build3DKozijn(scene, $currentKozijn, $currentGeometry);
    }
  });

  // --- Lifecycle ---

  let resizeObserver;

  onMount(async () => {
    const ok = await loadThreeJS();
    if (ok) {
      initScene();
      if ($currentKozijn && $currentGeometry) {
        build3DKozijn(scene, $currentKozijn, $currentGeometry);
      }
      // Watch for container resize
      resizeObserver = new ResizeObserver(() => handleResize());
      resizeObserver.observe(container);
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
