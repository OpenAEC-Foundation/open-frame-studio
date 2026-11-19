<script>
  import { onMount, onDestroy, untrack } from "svelte";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { allProfiles } from "../../stores/profiles.js";
  import { buildFrameGroup } from "./frameGroup.js";
  import { inspectTimberJoints } from "./tenonMesh.js";
  import { _ } from "svelte-i18n";

  let { visible = true } = $props();

  let container = $state(null);
  let canvas = $state(null);
  let THREE = $state(null);
  let loading = $state(true);
  let loadError = $state(false);
  let inspecting = $state(false);
  let hasJoints = $derived($currentGeometry?.archJoints?.some(joint => joint.tenon)
    || $currentGeometry?.curvedDividerTimber?.some(p => p.start?.tenon || p.end?.tenon));

  function toggleJoinery() {
    inspecting = !inspecting;
    if (kozijnGroup) inspectTimberJoints(kozijnGroup, inspecting, $currentKozijn?.frame?.frameDepth || 114);
  }

  // Three.js objects
  let renderer, scene, camera;
  let animationId;
  let kozijnGroup = null;

  // Camera orbit state
  let isDragging = false;
  let previousMouse = { x: 0, y: 0 };
  let spherical = { theta: Math.PI / 4, phi: Math.PI / 3, radius: 2500 };
  let target = { x: 0, y: 0, z: 0 };

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
    if (typeof window !== "undefined") window.__OFS_SCENE__ = scene;

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

  function build3DKozijn(scene, kozijn, geometry, profiles) {
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

    kozijnGroup = buildFrameGroup(THREE, kozijn, geometry, profiles);

    // Center the model
    const bbox = new THREE.Box3().setFromObject(kozijnGroup);
    const center = bbox.getCenter(new THREE.Vector3());
    kozijnGroup.position.sub(center);

    // Shift up so the bottom sits near the grid
    const size = bbox.getSize(new THREE.Vector3());
    kozijnGroup.position.y += size.y / 2;

    scene.add(kozijnGroup);
    if (untrack(() => inspecting)) inspectTimberJoints(kozijnGroup, true, kozijn?.frame?.frameDepth || 114);

    // Adjust camera distance based on kozijn size
    const maxDim = Math.max(size.x, size.y, size.z);
    spherical.radius = maxDim * 1.8;
    target = { x: 0, y: size.y / 2, z: 0 };
    updateCameraPosition();
  }

  // --- Debug hook (numerieke verificatie 2D↔3D) ---
  // window.__OFS_3D_SUMMARY__() geeft per mesh {role, bbox} in mm-model-
  // coördinaten (2D x; y omhoog vanaf de bovenrand, dus 0..-hoogte; z diepte,
  // -=buiten). Leeft op de actuele kozijnGroup en is dus na elke rebuild vers.
  function sceneSummary() {
    const meshes = [];
    if (kozijnGroup && THREE) {
      kozijnGroup.updateMatrixWorld(true);
      const gp = kozijnGroup.position;
      const box = new THREE.Box3();
      kozijnGroup.traverse((child) => {
        if (!child.isMesh) return;
        box.setFromObject(child);
        if (box.isEmpty()) return;
        const r1 = (v) => Math.round(v * 10) / 10;
        meshes.push({
          role: child.userData?.role || "unknown",
          ...(child.userData.timber ? { timber: child.userData.timber } : {}),
          ...(child.userData.divider ? { divider: child.userData.divider } : {}),
          ...(child.userData.joinery ? { joinery: child.userData.joinery } : {}),
          ...(child.userData.leaf ? { leaf: child.userData.leaf } : {}),
          bbox: {
            x: r1(box.min.x - gp.x),
            y: r1(box.min.y - gp.y),
            z: r1(box.min.z - gp.z),
            w: r1(box.max.x - box.min.x),
            h: r1(box.max.y - box.min.y),
            d: r1(box.max.z - box.min.z),
          },
        });
      });
    }
    return { meshes };
  }

  // --- Reactive rebuild ---
  // Rebuild on store changes, including when a custom contour is edited in place
  // without adding vertices or changing its id.
  $effect(() => {
    const k = $currentKozijn, g = $currentGeometry, profiles = $allProfiles;
    if (scene && k && g) build3DKozijn(scene, k, g, profiles || []);
  });

  // --- Lifecycle ---

  let resizeObserver;

  let threeLoaded = $state(false);

  onMount(async () => {
    if (typeof window !== "undefined") window.__OFS_3D_SUMMARY__ = sceneSummary;
    threeLoaded = await loadThreeJS();
  });

  // Initialize scene when visible becomes true AND Three.js is loaded
  $effect(() => {
    if (visible && threeLoaded && container && canvas && !renderer) {
      try {
        initScene();
        if ($currentKozijn && $currentGeometry) {
          build3DKozijn(scene, $currentKozijn, $currentGeometry, $allProfiles || []);
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
    if (typeof window !== "undefined") {
      if (window.__OFS_3D_SUMMARY__ === sceneSummary) delete window.__OFS_3D_SUMMARY__;
      if (window.__OFS_SCENE__ === scene) delete window.__OFS_SCENE__;
    }
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
    role="region"
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
    {#if hasJoints && !loading && !loadError}
      <button class="joint-view" aria-pressed={inspecting} onmousedown={(e) => e.stopPropagation()} onclick={toggleJoinery}>
        {$_("viewer3d.inspectTenons")}
      </button>
    {/if}
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
  .joint-view {
    position: absolute; top: 12px; right: 12px;
    padding: 6px 10px; border: 1px solid var(--border-color);
    border-radius: 4px; background: var(--bg-surface); color: var(--text-primary);
    cursor: pointer;
  }
  .joint-view[aria-pressed="true"] { border-color: var(--amber); }

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
