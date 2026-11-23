<script>
  import { onMount, onDestroy, untrack } from "svelte";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { allProfiles } from "../../stores/profiles.js";
  import { buildFrameGroup } from "./frameGroup.js";
  import { inspectTimberJoints } from "./tenonMesh.js";
  import { sectionCapsAt } from "./systemSection.js";
  import ViewCube from "./ViewCube.svelte";
  import { VIEW_FOV, HOME_DIRECTION, anglesForDirection, orbitOrientation, dragOrbit, snapOrbit, fitDistance, applyViewerCamera } from "./viewerCamera.js";
  import { _ } from "svelte-i18n";

  let { visible = true } = $props();

  let container = $state(null);
  let canvas = $state(null);
  let THREE = $state(null);
  let loading = $state(true);
  let loadError = $state(false);
  let inspecting = $state(false);
  let sectionOn = $state(false);
  let sectionCaps;
  const systemFrame = $derived(["pvc","aluminum"].includes($currentKozijn?.frame.material));
  let hasJoints = $derived($currentGeometry?.archJoints?.some(joint => joint.tenon)
    || $currentGeometry?.curvedDividerTimber?.some(p => p.start?.tenon || p.end?.tenon));

  function toggleJoinery() {
    inspecting = !inspecting;
    if (kozijnGroup) inspectTimberJoints(kozijnGroup, inspecting, $currentKozijn?.frame?.frameDepth || 114);
    requestRender();
  }

  function showSection(active, height) {
    renderer.clippingPlanes = [];
    if (sectionCaps) sectionCaps.visible = active;
    if (!active || !kozijnGroup) return;
    const cut = height/2 + 0.07;
    sectionCaps ??= sectionCapsAt(THREE,kozijnGroup,cut);
    if (!sectionCaps.parent) kozijnGroup.add(sectionCaps);
    renderer.clippingPlanes = [new THREE.Plane(new THREE.Vector3(0,-1,0),kozijnGroup.position.y-cut+0.01)];
  }
  function toggleSection() {
    sectionOn = !sectionOn;
    showSection(sectionOn,$currentKozijn.frame.outerHeight);
    requestRender();
  }

  // Three.js objects
  let renderer, scene, camera, perspectiveCamera, orthoCamera;
  let animationId;
  let kozijnGroup = null;

  let angles = anglesForDirection(HOME_DIRECTION);
  let orientation = $state.raw(orbitOrientation(angles));
  let orthographic = $state(false);
  let activeSnap, pointer;
  let distance = 2500, aspect = 1, boundRadius = 1000;
  let target = [0, 0, 0];
  let framedId, autoFit = true;

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

    // WebGL context options are fixed on the first call to getContext.
    // Let Three request the antialiased context and report initialization errors.
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
    perspectiveCamera = new THREE.PerspectiveCamera(VIEW_FOV);
    orthoCamera = new THREE.OrthographicCamera();
    camera = perspectiveCamera;
    updateCamera();

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

    renderer.domElement.addEventListener('webglcontextrestored', requestRender);
  }

  function requestRender() {
    if (renderer && !animationId) animationId = requestAnimationFrame(renderFrame);
  }

  function renderFrame(now) {
    animationId = null;
    if (activeSnap) {
      const progress = (now - activeSnap.start) / 240;
      setAngles(snapOrbit(activeSnap.from, activeSnap.to, progress), false);
      if (progress >= 1) activeSnap = null;
    }
    renderer.render(scene, camera);
    if (activeSnap) requestRender();
  }

  function updateCamera(redraw = true) {
    if (!camera) return;
    applyViewerCamera(camera, orientation, target, distance, aspect, boundRadius);
    if (redraw) requestRender();
  }

  function setAngles(next, redraw = true) {
    angles = next;
    orientation = orbitOrientation(next);
    updateCamera(redraw);
  }

  function selectView(direction) {
    const to = anglesForDirection(direction);
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
      activeSnap = null;
      setAngles(to);
    } else {
      activeSnap = { from: angles, to, start: performance.now() };
      requestRender();
    }
  }

  function homeView() {
    autoFit = true;
    distance = fitDistance(boundRadius, aspect);
    updateCamera();
    selectView(HOME_DIRECTION);
  }

  function toggleProjection() {
    orthographic = !orthographic;
    camera = orthographic ? orthoCamera : perspectiveCamera;
    updateCamera();
  }

  function handleResize() {
    if (!container || !renderer || !camera) return;
    const w = container.clientWidth;
    const h = container.clientHeight;
    if (w === 0 || h === 0) return;
    renderer.setSize(w, h);
    aspect = w / h;
    if (autoFit) distance = fitDistance(boundRadius, aspect);
    updateCamera();
  }

  function orbitBy(dx, dy) {
    activeSnap = null;
    setAngles(dragOrbit(angles, dx, dy));
  }

  function onPointerDown(e) {
    if (e.button !== 0 || pointer) return;
    activeSnap = null;
    pointer = { id: e.pointerId, x: e.clientX, y: e.clientY };
    canvas.setPointerCapture(e.pointerId);
    e.preventDefault();
  }

  function onPointerMove(e) {
    if (e.pointerId !== pointer?.id) return;
    orbitBy(e.clientX - pointer.x, e.clientY - pointer.y);
    pointer.x = e.clientX;
    pointer.y = e.clientY;
  }

  function onPointerEnd(e) {
    if (e.pointerId === pointer?.id) pointer = null;
  }

  function onWheel(e) {
    e.preventDefault();
    if (!e.deltaY) return;
    autoFit = false;
    const fitted = fitDistance(boundRadius, aspect);
    distance = Math.max(fitted / 100, Math.min(fitted * 15, distance * 1.1 ** Math.sign(e.deltaY)));
    updateCamera();
  }

  // --- 3D Kozijn Builder ---

  function clearFrame() {
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
      kozijnGroup = null;
    }
    sectionCaps = null;
    renderer.clippingPlanes = [];
  }

  function build3DKozijn(scene, kozijn, geometry, profiles) {
    clearFrame();
    if (!kozijn || !geometry) {
      framedId = null;
      activeSnap = null;
      requestRender();
      return;
    }

    kozijnGroup = buildFrameGroup(THREE, kozijn, geometry, profiles);

    // Center the model
    const bbox = new THREE.Box3().setFromObject(kozijnGroup);
    const center = bbox.getCenter(new THREE.Vector3());
    kozijnGroup.position.sub(center);

    // Shift up so the bottom sits near the grid
    const size = bbox.getSize(new THREE.Vector3());
    kozijnGroup.position.y += size.y / 2;

    scene.add(kozijnGroup);
    if (untrack(()=>sectionOn) && ["aluminum","pvc"].includes(kozijn.frame.material)) showSection(true,kozijn.frame.outerHeight);
    if (untrack(() => inspecting)) inspectTimberJoints(kozijnGroup, true, kozijn?.frame?.frameDepth || 114);

    if (kozijn.id !== framedId) {
      framedId = kozijn.id;
      autoFit = true;
    }
    boundRadius = Math.max(1, size.length() / 2);
    if (autoFit) distance = fitDistance(boundRadius, aspect);
    target = [0, size.y / 2, 0];
    updateCamera();
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
          ...(child.userData.systemSeal ? { systemSeal: child.userData.systemSeal } : {}),
          ...(child.userData.hardware ? { hardware: child.userData.hardware } : {}),
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
    return { meshes, camera: camera && { position: camera.position.toArray(), up: camera.up.toArray(),
      target: [...target], distance, projection: camera.isOrthographicCamera ? 'orthographic' : 'perspective' } };
  }

  // --- Reactive rebuild ---
  // Rebuild on store changes, including when a custom contour is edited in place
  // without adding vertices or changing its id.
  $effect(() => {
    const k = $currentKozijn, g = $currentGeometry, profiles = $allProfiles;
    if (renderer) untrack(() => build3DKozijn(scene, k, g, profiles || []));
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
        untrack(initScene);
        if (!renderer) return;
        if ($currentKozijn && $currentGeometry) {
          untrack(() => build3DKozijn(scene, $currentKozijn, $currentGeometry, $allProfiles || []));
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
      renderer.domElement.removeEventListener('webglcontextrestored', requestRender);
      clearFrame();
      renderer.dispose();
      renderer = null;
    }
    scene = null;
    camera = null;
  });
</script>

{#if visible}
  <div
    class="viewer3d-container"
    bind:this={container}
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

    <canvas bind:this={canvas} onpointerdown={onPointerDown} onpointermove={onPointerMove}
      onpointerup={onPointerEnd} onlostpointercapture={onPointerEnd} onpointercancel={onPointerEnd} onwheel={onWheel}></canvas>
    {#if !loading && !loadError && $currentKozijn}
      <ViewCube {orientation} {orthographic} onselect={selectView} onorbit={orbitBy} onhome={homeView} ontoggleprojection={toggleProjection} />
    {/if}
    {#if hasJoints && !loading && !loadError}
      <button class="joint-view" aria-pressed={inspecting} onmousedown={(e) => e.stopPropagation()} onclick={toggleJoinery}>
        {$_("viewer3d.inspectTenons")}
      </button>
    {:else if systemFrame && !loading && !loadError}
      <button class="joint-view" aria-pressed={sectionOn} onmousedown={(e) => e.stopPropagation()} onclick={toggleSection}>
        {$_("viewer3d.section")}
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
    touch-action: none;
    cursor: grab;
  }
  canvas:active { cursor: grabbing; }
  .joint-view {
    position: absolute; top: 12px; left: 12px;
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
