<script>
  import { onMount, onDestroy } from "svelte";
  import { currentKozijn, currentGeometry } from "../../stores/kozijn.js";
  import { allProfiles } from "../../stores/profiles.js";
  import { ralToHex } from "../../lib/ral-colors.js";
  import {
    ringBBox, miterEndsFor, layoutHasBuiten, collectBuitenRects,
    expandBuitenRect, decomposeSteppedFrame, rectAdjacency,
    dividerSectionContour, sectionFromPoints,
  } from "../../lib/frame3d.js";
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
  const MASONRY_COLOR = 0x9C876D; // matte baksteen (borstwering onder 'buiten'-vak)
  const SASH_PROUD_MM = 8; // vleugel-binnenvlak steekt iets voor het kozijn (opdek)
  const GLASS_EDGE_CLEARANCE_MM = 5; // speling glas t.o.v. sponningbodem per zijde
  const SEAL_EPS = 0.5; // mm overlap op contactvlakken tegen coplanaire z-fighting

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
    // #8: prefer the chosen RAL colour so the picked colour shows in 3D too;
    // fall back to the material's natural tint when no RAL is set.
    const ral = kozijn?.frame?.colorInside;
    if (ral) {
      const hex = ralToHex(ral);
      if (typeof hex === "string" && hex[0] === "#") return parseInt(hex.slice(1), 16);
    }
    const key = getMaterialKey(kozijn?.frame?.material);
    return MATERIAL_COLORS[key] || MATERIAL_COLORS["wood(meranti)"];
  }

  // Compact string hash (djb2) — rebuild-signature van grotere deelobjecten
  // (layout-boom, geometrie-payload) zonder de volledige JSON te bewaren.
  function hashStr(s) {
    let h = 5381;
    for (let i = 0; i < s.length; i++) h = ((h << 5) + h + s.charCodeAt(i)) | 0;
    return (h >>> 0).toString(36);
  }

  // --- Profiel-doorsnedes (echte kozijnhout-/kamerprofiel-secties) ---
  //
  // crossSection-conventie van de profielbibliotheek (profiles/*.json en
  // ui/src/lib/profileContour.js): [u, v]-punten in mm.
  //   u: 0 = muurzijde ..... w = vakzijde (dagkant, waar het glas/vak zit)
  //   v: 0 = buitenzijde ... d = binnenzijde (bouwdiepte)
  // Klopt met de sponning-metadata op schijf: een "binnensponning" heeft zijn
  // inkeping op de vakzijde-binnen-hoek (u=w, v=d), een "buitensponning" op
  // (u=w, v=0). NB: de embedded browser-fallback in stores/profiles.js
  // (generateCrossSection) is ouder en u-symmetrisch; die rendert hiermee ook,
  // alleen kan zijn sponningdetail aan de andere dieptezijde uitkomen.
  //
  // Een bruikbare doorsnede heeft na opschoning ≥ 5 punten: de bibliotheek-
  // kozijnhoutsecties (één sponninginkeping) tellen er 6; rechthoekige
  // placeholders (glaslat, spouwlat) tellen er 4 en renderen als box —
  // identiek aan een rechthoek-extrusie.
  // Lookup identiek aan PropertiesPanel: allProfiles bevat bibliotheek +
  // custom/geïmporteerde profielen (project.custom_profiles).
  function resolveProfileSection(ref, profiles) {
    if (!ref?.id || !Array.isArray(profiles)) return null;
    const def = profiles.find((p) => p.id === ref.id);
    const cs = def?.crossSection;
    if (!Array.isArray(cs) || cs.length < 5) return null;
    const pts = [];
    for (const p of cs) {
      const u = Number(p?.[0]);
      const v = Number(p?.[1]);
      if (!Number.isFinite(u) || !Number.isFinite(v)) return null;
      // Opeenvolgende dubbele punten overslaan (DXF-import herhaalt punten).
      const prev = pts[pts.length - 1];
      if (prev && Math.abs(prev[0] - u) < 1e-6 && Math.abs(prev[1] - v) < 1e-6) continue;
      pts.push([u, v]);
    }
    // Expliciet sluitpunt (gelijk aan startpunt) weglaten.
    if (pts.length > 2) {
      const [fu, fv] = pts[0];
      const [lu, lv] = pts[pts.length - 1];
      if (Math.abs(fu - lu) < 1e-6 && Math.abs(fv - lv) < 1e-6) pts.pop();
    }
    if (pts.length < 5) return null;
    let minU = Infinity, maxU = -Infinity, minV = Infinity, maxV = -Infinity;
    for (const [u, v] of pts) {
      if (u < minU) minU = u;
      if (u > maxU) maxU = u;
      if (v < minV) minV = v;
      if (v > maxV) maxV = v;
    }
    const w = maxU - minU;
    const d = maxV - minV;
    if (!(w > 0.5) || !(d > 0.5)) return null;
    // `def` rijdt mee voor sponningmetadata (glas-z in de sponning).
    return { id: def.id, def, pts, minU, minV, w, d };
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

    // Helper: role-tag voor de debug-samenvatting (__OFS_3D_SUMMARY__)
    function tag(mesh, role) {
      if (mesh) mesh.userData.role = role;
      return mesh;
    }

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
    function addBorder(rect, memberWidth, depth, material, zOffset = 0, role = "sash") {
      const w = Math.min(memberWidth, rect.width / 2, rect.height / 2);
      const innerH = Math.max(1, rect.height - 2 * w);
      kozijnGroup.add(tag(makeBox({ x: rect.x, y: rect.y, width: rect.width, height: w }, depth, material, zOffset), role));
      kozijnGroup.add(tag(makeBox({ x: rect.x, y: rect.y + rect.height - w, width: rect.width, height: w }, depth, material, zOffset), role));
      kozijnGroup.add(tag(makeBox({ x: rect.x, y: rect.y + w, width: w, height: innerH }, depth, material, zOffset), role));
      kozijnGroup.add(tag(makeBox({ x: rect.x + rect.width - w, y: rect.y + w, width: w, height: innerH }, depth, material, zOffset), role));
    }

    // --- Echte profiel-extrusies voor kozijnleden ---
    // Hergebruik binnen deze build: één THREE.Shape per profiel en één
    // ExtrudeGeometry per uniek (profiel, lengte)-paar — vakken met gelijke
    // maten delen zo hun geometrie. Meshes mogen geometrie delen: de teardown
    // in build3DKozijn/onDestroy mag dispose() dan meermaals aanroepen
    // (BufferGeometry.dispose is idempotent).
    const sectionCache = new Map(); // profileRef.id -> section | null
    const shapeCache = new Map(); // section.id -> THREE.Shape
    const extrudeCache = new Map(); // `${section.id}|${lengte}` -> ExtrudeGeometry
    const profileList = Array.isArray(profiles) ? profiles : [];

    function getSection(ref) {
      const key = ref?.id;
      if (!key) return null;
      if (sectionCache.has(key)) return sectionCache.get(key);
      const section = resolveProfileSection(ref, profileList);
      sectionCache.set(key, section);
      return section;
    }

    function getExtrudedGeometry(section, length, mirrored) {
      const shapeKey = mirrored ? `${section.id}|m` : section.id;
      const key = `${shapeKey}|${length}`;
      let geo = extrudeCache.get(key);
      if (!geo) {
        let shape = shapeCache.get(shapeKey);
        if (!shape) {
          // Gespiegeld: u' = w - u (vakzijde wisselt van kant); ExtrudeGeometry
          // normaliseert de omgekeerde winding zelf.
          const su = (u) => (mirrored ? section.w - (u - section.minU) : u - section.minU);
          shape = new THREE.Shape();
          shape.moveTo(su(section.pts[0][0]), section.pts[0][1] - section.minV);
          for (let i = 1; i < section.pts.length; i++) {
            shape.lineTo(su(section.pts[i][0]), section.pts[i][1] - section.minV);
          }
          shape.closePath();
          shapeCache.set(shapeKey, shape);
        }
        geo = new THREE.ExtrudeGeometry(shape, { depth: length, bevelEnabled: false });
        extrudeCache.set(key, geo);
      }
      return geo;
    }

    // Eén kozijnlid als profiel-extrusie.
    //
    // Viewer-assen: X = breedte (2D x), Y = hoogte (2D y omgeklapt), Z = diepte
    // met -Z = buiten en +Z = binnen (glas en muur zitten richting -Z). De
    // shape leeft lokaal in XY (x = u = aanzichtbreedte, y = v = bouwdiepte,
    // v=0 = buiten) en wordt langs lokaal +Z over de lidlengte geëxtrudeerd.
    // De rotaties (Euler-order ZYX, numeriek geverifieerd tegen three r183):
    //   stijl  ("v"): (u, v, t) -> wereld (u, -t, v) — doorsnede in het
    //                 XZ-vlak, extrusie langs Y; u loopt naar +X
    //   dorpel ("h"): (u, v, t) -> wereld (t, u, v)  — doorsnede in het
    //                 YZ-vlak, extrusie langs X; u loopt naar +Y
    // `mirrored` klapt de doorsnede in de aanzichtrichting zodat de vakzijde
    // (u=w) naar het vak wijst: linkerstijl/onderdorpel normaal,
    // rechterstijl/bovendorpel gespiegeld; tussenleden (glas aan twee kanten)
    // blijven ongespiegeld/gecentreerd.
    // Binnenvlakken liggen vlak in één front: v=d (binnen) op Z=+frameDepth/2,
    // zodat een dieper profiel (bv. onderdorpel met waterhol) aan de
    // buitenzijde uitsteekt — zoals in het echt. De doorsnede wordt in de
    // aanzichtrichting gecentreerd op het lid als profiel- en lidbreedte
    // verschillen. Leden sluiten stomp aan (geen verstek) — zelfde aansluiting
    // als de oude boxen; verstekken volgen in een latere fase.
    // opts: { zInner, side, miterStart, miterEnd }
    //   zInner     — wereld-z van het binnenvlak (v=d); default frameDepth/2.
    //   side       — 'top'|'bottom'|'left'|'right': welke buitenrand het lid
    //                heeft (nodig om de verstekrichting te bepalen).
    //   miterStart — 45°-verstek op het startvlak van de extrusie (links voor
    //                "h", boven voor "v"); miterEnd idem op het eindvlak.
    // Het verstek wordt geschoren op een kloon van de extrusie: eindkap-
    // vertices schuiven langs de lid-as evenredig met hun afstand tot de
    // buitenrand, zodat twee versteklede elkaar exact op de 45°-diagonaal
    // ontmoeten (zoals de framePolygons uit de payload).
    function makeProfileMember(rect, section, material, orientation, mirrored = false, opts = {}) {
      const length = orientation === "v" ? rect.height : rect.width;
      if (!(length > 0) || !(rect.width > 0) || !(rect.height > 0)) return null;
      const zInner = typeof opts.zInner === "number" ? opts.zInner : frameDepth / 2;
      let geo = getExtrudedGeometry(section, length, mirrored);
      if (opts.miterStart || opts.miterEnd) {
        geo = geo.clone();
        const crossW = orientation === "v" ? rect.width : rect.height;
        const off = (crossW - section.w) / 2;
        const outerDist = (a) =>
          opts.side === "top" || opts.side === "right" ? crossW - off - a : off + a;
        const pos = geo.getAttribute("position");
        const maxShift = Math.min(crossW, length / 2);
        for (let i = 0; i < pos.count; i++) {
          const shift = Math.min(maxShift, Math.max(0, outerDist(pos.getX(i))));
          const z = pos.getZ(i);
          if (opts.miterStart && Math.abs(z) < 1e-3) pos.setZ(i, shift);
          else if (opts.miterEnd && Math.abs(z - length) < 1e-3) pos.setZ(i, length - shift);
        }
        pos.needsUpdate = true;
        geo.computeVertexNormals();
      }
      const mesh = new THREE.Mesh(geo, material);
      mesh.rotation.order = "ZYX"; // eerst de X-, dan de Z-rotatie op de vector
      const zBinnen = zInner - section.d; // v=0 hier → v=d op zInner
      if (orientation === "v") {
        mesh.rotation.set(Math.PI / 2, 0, 0);
        mesh.position.set(
          rect.x + (rect.width - section.w) / 2, // aanzicht gecentreerd op het lid
          -rect.y, // extrusie start bovenaan en loopt omlaag
          zBinnen
        );
      } else {
        mesh.rotation.set(Math.PI / 2, 0, Math.PI / 2);
        mesh.position.set(
          rect.x, // extrusie start links
          -(rect.y + rect.height) + (rect.height - section.w) / 2,
          zBinnen
        );
      }
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      return mesh;
    }

    const f = kozijn.frame || {};
    const fw = f.frameWidth || 67;
    const baseMat = matKey.startsWith("wood(") ? "wood" : matKey; // wood|pvc|aluminum|wood_aluminum
    const miterCorners = baseMat === "aluminum" || baseMat === "pvc";
    const layoutTree = kozijn.layout || null;
    const memberRefs = {
      top: f.topProfile || f.profile,
      bottom: f.bottomProfile || f.sillProfile || f.profile,
      left: f.leftProfile || f.profile,
      right: f.rightProfile || f.profile,
    };
    const memberDefaults = {
      top: { orientation: "h", mirrored: true },
      bottom: { orientation: "h", mirrored: false },
      left: { orientation: "v", mirrored: false },
      right: { orientation: "v", mirrored: true },
    };

    // Frame members — de payload is leidend (leden mogen elkaar niet
    // doorsnijden of gaten laten):
    //  1. Stepped (melkmeisje, layout met 'buiten'-vak): framePolygons zijn
    //     ringen per vak; decomponeer hun unie (min vakken/dividers) tot
    //     disjuncte leden. Zo bestaat de dorpel onder het zijlicht, loopt er
    //     geen lid door de borstweringzone en ontstaan geen dubbele
    //     coplanaire boxen (z-fighting).
    //  2. Per-lid polygonen (verstek / gemengde hoeken): volgorde [top,
    //     bottom, left, right]; lidlengte = bbox van de polygon, diagonale
    //     uiteinden worden als 45°-verstek geschoren.
    //  3. Anders: de klassieke frameRects (byte-identiek grid-pad).
    const framePolys = Array.isArray(geometry.framePolygons) ? geometry.framePolygons : [];
    const steppedFrame = framePolys.length > 0 && layoutTree && layoutHasBuiten(layoutTree);
    if (steppedFrame) {
      const ow = f.outerWidth || geometry.outerRect?.width || 0;
      const oh = f.outerHeight || geometry.outerRect?.height || 0;
      const vakRects = (geometry.cellRects || []).map((c) => c.rect);
      const holeRects = [
        ...vakRects,
        ...(geometry.vDividers || []),
        ...(geometry.hDividers || []),
      ];
      const rings = framePolys.map(ringBBox);
      for (const m of decomposeSteppedFrame(rings, holeRects)) {
        const orientation = m.width >= m.height ? "h" : "v";
        const adj = rectAdjacency(m, vakRects);
        let side;
        if (orientation === "h") {
          // Vak boven het lid (2D) → het gedraagt zich als onderdorpel;
          // vak eronder → bovendorpel. Sponning wijst zo naar het vak.
          if (adj.top && !adj.bottom) side = "bottom";
          else if (adj.bottom && !adj.top) side = "top";
          else side = m.y < oh / 2 ? "top" : "bottom";
        } else {
          if (adj.right && !adj.left) side = "left";
          else if (adj.left && !adj.right) side = "right";
          else side = m.x < ow / 2 ? "left" : "right";
        }
        const d = memberDefaults[side];
        const section = getSection(memberRefs[side]);
        const mesh = section && makeProfileMember(m, section, frameMat, orientation, d.mirrored, { side });
        kozijnGroup.add(tag(mesh || makeBox(m, frameDepth, frameMat), "frame-member"));
      }
    } else if (framePolys.length === 4) {
      ["top", "bottom", "left", "right"].forEach((side, i) => {
        const rect = ringBBox(framePolys[i]);
        if (!(rect.width > 0 && rect.height > 0)) return;
        const d = memberDefaults[side];
        const { start, end } = miterEndsFor(framePolys[i], d.orientation);
        const section = getSection(memberRefs[side]);
        const mesh = section && makeProfileMember(rect, section, frameMat, d.orientation, d.mirrored, {
          side, miterStart: start, miterEnd: end,
        });
        kozijnGroup.add(tag(mesh || makeBox(rect, frameDepth, frameMat), "frame-member"));
      });
    } else if (geometry.frameRects) {
      // frameRects-volgorde uit compute_2d_geometry: [top, bottom/sill, left,
      // right]; per lid geldt de override-keten uit PropertiesPanel. Spiegeling:
      // vakzijde van de doorsnede naar het vak toe (top en rechts gespiegeld).
      const sides = ["top", "bottom", "left", "right"];
      geometry.frameRects.forEach((rect, i) => {
        const side = sides[i];
        const m = side
          ? { ref: memberRefs[side], ...memberDefaults[side] }
          : { ref: f.profile, orientation: rect.height >= rect.width ? "v" : "h", mirrored: false };
        const section = getSection(m.ref);
        const mesh = section && makeProfileMember(rect, section, frameMat, m.orientation, m.mirrored);
        kozijnGroup.add(tag(mesh || makeBox(rect, frameDepth, frameMat), "frame-member"));
      });
    }

    // 'Buiten'-vakken (melkmeisje-borstwering): metselwerkvolume in plaats van
    // een zwart gat, doorgetrokken tot de buitenrand aan zijden zonder kozijn.
    if (layoutTree && geometry.innerRect && layoutHasBuiten(layoutTree)) {
      const masonryMat = new THREE.MeshStandardMaterial({
        color: MASONRY_COLOR, roughness: 0.95, metalness: 0.0,
      });
      for (const b of collectBuitenRects(layoutTree, geometry.innerRect, fw)) {
        const r = expandBuitenRect(b, geometry.innerRect, fw);
        // 1 mm terugliggend aan beide dieptezijden: geen coplanaire vlakken
        // met muur (achter) of kozijnbinnenvlak (voor).
        kozijnGroup.add(tag(makeBox(r, frameDepth - 2, masonryMat, 0), "masonry"));
      }
    }

    // Cell contents: glass, sash frames, panel fillings, glazing beads
    // Glass sits in the sponning (rebate), offset toward the outside (-Z).
    const GLASS_CLEARANCE = 4; // mm clearance per side
    const glassThickness = kozijn.cells?.[0]?.glazing?.thicknessMm || 24;
    const glassZOffset = frameDepth * 0.3 - frameDepth / 2;

    // Sponning-/glaslatmaten uit de payload (profielsnapshot); klassieke
    // fallbacks voor kozijnen zonder snapshot.
    const sponningHoogte = typeof geometry.sponningHoogte === "number" ? geometry.sponningHoogte : 17;
    const beadView = typeof geometry.glaslatHoogte === "number" ? geometry.glaslatHoogte : 17;
    const beadFoot = typeof geometry.glaslatBreedte === "number" ? geometry.glaslatBreedte : 15;

    // Diepteligging (v) van de sponningbodem in een doorsnede: hout heeft zijn
    // glassponning aan de binnenzijde (v = d − sponningbreedte, zie
    // profileContour.js), PVC/alu een glaskanaal vanaf de aanslag (~25 mm
    // achter het buitenvlak, bodyContour).
    function rebateLedge(d, def) {
      const mat = def?.material || baseMat;
      if (mat === "wood" || mat === "wood_aluminum") {
        const m = def?.sponning?.width || def?.glazingRebate || 51;
        return Math.max(4, d - m);
      }
      return Math.min(25, d * 0.4);
    }

    // Glas ín de sponning van zijn drager (kozijn of vleugel): tegen de
    // aanslag, maat = dagmaat + 2×(glasinval − speling), glaslat tegen het
    // glas aan de binnenzijde. SEAL_EPS voorkomt coplanaire contactvlakken.
    function addGlassPane(dayRect, hostSection, hostZInner, thickness, beadPosition) {
      const d = hostSection?.d ?? frameDepth;
      const outerZ = hostZInner - d + rebateLedge(d, hostSection?.def) + SEAL_EPS;
      const expand = Math.max(0, sponningHoogte - GLASS_EDGE_CLEARANCE_MM);
      kozijnGroup.add(tag(makeBox(insetRectBy(dayRect, -expand), thickness, glassMat, outerZ + thickness / 2), "glass"));
      const beadRect = insetRectBy(dayRect, -sponningHoogte);
      const inside = beadPosition !== "buiten";
      const beadZ = inside
        ? outerZ + thickness - SEAL_EPS + beadFoot / 2
        : outerZ + SEAL_EPS - beadFoot / 2;
      addBorder(beadRect, beadView, beadFoot, beadMat, beadZ, "bead");
    }

    // Raamhout-/deurhoutdoorsnede voor vleugels: voorkeurs-ids uit de
    // bibliotheek (KVT-draaikiep 69x90 als raamdefault), anders generiek op
    // applicableAs + materiaal, anders het kozijnprofiel.
    function findSashSection(isDoor) {
      if (baseMat === "wood" || baseMat === "wood_aluminum") {
        const prefIds = isDoor
          ? ["deur-meranti-67x114", "deurhout-54x118", "deur-meranti-67x130"]
          : ["raam-meranti-69x90", "raamhout-69x90", "raamhout-54x78", "raam-meranti-54x78"];
        for (const id of prefIds) {
          const s = getSection({ id });
          if (s) return s;
        }
      }
      const want = isDoor ? ["deur_stijl", "sash"] : ["raam_stijl", "sash"];
      const cand = profileList.find((p) =>
        p.material === baseMat &&
        Array.isArray(p.applicableAs) && p.applicableAs.some((a) => want.includes(a)) &&
        Array.isArray(p.crossSection) && p.crossSection.length >= 5);
      if (cand) {
        const s = getSection({ id: cand.id });
        if (s) return s;
      }
      return getSection(f.profile) || null;
    }

    // Vleugelkader met echte doorsnede. Hout: stijlen lopen door, dorpels
    // ertussen (KVT); PVC/alu: vier leden in 45°-verstek. Retourneert de
    // wereld-z van het vleugel-binnenvlak (voor glas/paneel in de vleugel).
    function addSashFrame(rect, section, sashWidth, miter) {
      const w = Math.min(sashWidth, rect.width / 2, rect.height / 2);
      if (!section) {
        addBorder(rect, w, frameDepth * 0.82, sashMat, frameDepth * 0.06, "sash");
        return frameDepth * 0.06 + frameDepth * 0.41;
      }
      const zInner = frameDepth / 2 + SASH_PROUD_MM;
      const members = miter
        ? [
            { r: { x: rect.x, y: rect.y, width: rect.width, height: w }, side: "top", ms: true, me: true },
            { r: { x: rect.x, y: rect.y + rect.height - w, width: rect.width, height: w }, side: "bottom", ms: true, me: true },
            { r: { x: rect.x, y: rect.y, width: w, height: rect.height }, side: "left", ms: true, me: true },
            { r: { x: rect.x + rect.width - w, y: rect.y, width: w, height: rect.height }, side: "right", ms: true, me: true },
          ]
        : [
            { r: { x: rect.x, y: rect.y, width: w, height: rect.height }, side: "left" },
            { r: { x: rect.x + rect.width - w, y: rect.y, width: w, height: rect.height }, side: "right" },
            { r: { x: rect.x + w, y: rect.y, width: Math.max(1, rect.width - 2 * w), height: w }, side: "top" },
            { r: { x: rect.x + w, y: rect.y + rect.height - w, width: Math.max(1, rect.width - 2 * w), height: w }, side: "bottom" },
          ];
      for (const m of members) {
        const d = memberDefaults[m.side];
        const mesh = makeProfileMember(m.r, section, sashMat, d.orientation, d.mirrored, {
          zInner, side: m.side, miterStart: !!m.ms, miterEnd: !!m.me,
        });
        if (mesh) kozijnGroup.add(tag(mesh, "sash"));
      }
      return zInner;
    }

    // Vak op het vrije-indeling-pad: details komen uit cellRect.vulling
    // (cellIndex is een layout-leaf-index, géén index in het vestigiale
    // kozijn.cells-matrixgrid).
    function renderLayoutCell(cellRect) {
      const v = cellRect.vulling || {};
      const type = typeof v.type === "string" ? v.type : "glas";
      if (type === "buiten") return; // metselwerk komt uit het layout-blok
      const rect = cellRect.rect;
      if (type === "paneel" || type === "rooster") {
        const inset = insetRectBy(rect, GLASS_CLEARANCE);
        const thickness = frameDepth * 0.6;
        const tint = type === "rooster" ? FILLING_COLORS.ventilation : PANEL_COLOR;
        const fillMat = new THREE.MeshStandardMaterial({ color: tint, roughness: 0.8, metalness: 0.0 });
        kozijnGroup.add(tag(makeBox(inset, thickness, fillMat, 0), "panel"));
        if (type === "rooster") {
          const slatCount = Math.max(2, Math.min(8, Math.floor(inset.height / 90)));
          const slatMat = new THREE.MeshStandardMaterial({ color: 0x6B7178, roughness: 0.5, metalness: 0.3 });
          const gap = inset.height / (slatCount + 1);
          for (let s = 1; s <= slatCount; s++) {
            kozijnGroup.add(tag(makeBox(
              { x: inset.x + 6, y: inset.y + gap * s - 4, width: Math.max(1, inset.width - 12), height: 8 },
              6, slatMat, thickness / 2 + 3
            ), "panel"));
          }
        }
        return;
      }
      const beadPos = v.glaslat === "buiten" ? "buiten" : "binnen";
      if (type === "glas") {
        const frameSection = getSection(memberRefs.left) || getSection(f.profile);
        addGlassPane(rect, frameSection, frameDepth / 2, glassThickness, beadPos);
        return;
      }
      // raam / deur — vleugel(s) met echte raamhout-/deurhoutdoorsnede.
      // Aanzichtbreedte: raam 69 default (KVT-draaikiep), deur = kozijnhout-
      // breedte (consistent met production.rs).
      const isDoor = type === "deur";
      const sashSection = findSashSection(isDoor);
      const sw = isDoor ? fw : (sashSection?.w || 69);
      const isStolp = isDoor && v.doorKind === "dubbel";
      const leaves = isStolp
        ? [
            { x: rect.x, y: rect.y, width: rect.width / 2, height: rect.height },
            { x: rect.x + rect.width / 2, y: rect.y, width: rect.width / 2, height: rect.height },
          ]
        : [rect];
      for (const leafRect of leaves) {
        const zInner = addSashFrame(leafRect, sashSection, sw, miterCorners);
        const opening = insetRectBy(leafRect, Math.min(sw, leafRect.width / 2, leafRect.height / 2));
        if (isDoor) {
          // Dicht deurblad in de vleugelsponning
          const d = sashSection?.d ?? frameDepth;
          const ledge = rebateLedge(d, sashSection?.def);
          const outerZ = zInner - d + ledge + SEAL_EPS;
          const thick = Math.min(54, Math.max(20, d - ledge - 10));
          const expand = Math.max(0, sponningHoogte - GLASS_EDGE_CLEARANCE_MM);
          kozijnGroup.add(tag(makeBox(insetRectBy(opening, -expand), thick, doorMat, outerZ + thick / 2), "door"));
        } else {
          addGlassPane(opening, sashSection, zInner, glassThickness, beadPos);
        }
      }
      if (isStolp) {
        // Stolpnaald: dun dekkend lid over de naad — géén vaste tussenstijl
        const naald = { x: rect.x + rect.width / 2 - 20, y: rect.y, width: 40, height: rect.height };
        kozijnGroup.add(tag(makeBox(naald, 15, sashMat, frameDepth / 2 + SASH_PROUD_MM + 7.5), "sash"));
      }
    }

    if (geometry.cellRects) {
      for (const cellRect of geometry.cellRects) {
        // Vrije indeling: vakdetails reizen mee op cellRect.vulling. Oude
        // wasm-bundles en matrix-kozijnen leveren het niet mee — dan bepaalt
        // de matrix-cel (kozijn.cells) het vaktype, exact zoals voorheen.
        if (cellRect.vulling && typeof cellRect.vulling.type === "string") {
          renderLayoutCell(cellRect);
          continue;
        }
        const cell = kozijn.cells?.[cellRect.cellIndex];
        const panelType = cell?.panelType || "fixed_glass";
        let kind;
        if (panelType === "panel") kind = "panel";
        else if (panelType === "ventilation") kind = "ventilation";
        else if (panelType === "door") kind = "door";
        else if (OPERABLE_TYPES.has(panelType)) kind = "sash";
        else kind = "glass";

        const rect = cellRect.rect;
        const cellInset = insetRectBy(rect, GLASS_CLEARANCE);

        // Vakvulling (infill panel / ventilation grille)
        if (kind === "panel" || kind === "ventilation") {
          const filling = cell?.panelFilling;
          const thickness = filling?.thicknessMm || frameDepth * 0.6;
          const tint = FILLING_COLORS[filling?.fillingType] ||
            (kind === "ventilation" ? FILLING_COLORS.ventilation : PANEL_COLOR);
          const fillMat = new THREE.MeshStandardMaterial({ color: tint, roughness: 0.8, metalness: 0.0 });
          // #6: an explicit inzet/diepte (setbackMm) shifts the fill along the
          // depth axis so different vakvullingen can sit at different depths;
          // null/undefined keeps the historic centred placement.
          const panelZ = typeof filling?.setbackMm === "number" ? filling.setbackMm : 0;
          kozijnGroup.add(tag(makeBox(cellInset, thickness, fillMat, panelZ), "panel"));

          // Ventilation grille — horizontal louver slats proud of the panel
          if (kind === "ventilation") {
            const slatCount = Math.max(2, Math.min(8, Math.floor(cellInset.height / 90)));
            const slatMat = new THREE.MeshStandardMaterial({ color: 0x6B7178, roughness: 0.5, metalness: 0.3 });
            const gap = cellInset.height / (slatCount + 1);
            for (let s = 1; s <= slatCount; s++) {
              kozijnGroup.add(tag(makeBox(
                { x: cellInset.x + 6, y: cellInset.y + gap * s - 4, width: Math.max(1, cellInset.width - 12), height: 8 },
                6, slatMat, thickness / 2 + 3
              ), "panel"));
            }
          }
          continue;
        }

        // Glazed cell (fixed glass, operable sash, or door)
        const isOperable = kind === "sash" || kind === "door";
        let glassHostRect = cellInset;

        if (isOperable) {
          // Sash frame: a border of sash_width around the cell opening
          const sashWidth = cell?.sashWidth || 67;
          const sashDepth = frameDepth * 0.82;
          const sashZ = frameDepth * 0.06; // slightly proud on the inside
          addBorder(cellInset, sashWidth, sashDepth, sashMat, sashZ);
          glassHostRect = insetRectBy(cellInset, sashWidth);
        }

        if (kind === "door") {
          // Door leaf — opaque infill inside the door sash frame
          kozijnGroup.add(tag(makeBox(glassHostRect, frameDepth * 0.7, doorMat, 0), "door"));
        } else {
          // Glass pane
          const thisGlassThickness = cell?.glazing?.thicknessMm || glassThickness;
          kozijnGroup.add(tag(makeBox(glassHostRect, thisGlassThickness, glassMat, glassZOffset), "glass"));

          // Glaslat (glazing beads) — border around the glass, inside or outside
          const gl = cell?.glaslat;
          if (gl) {
            const beadWidth = gl.widthMm || 15;
            const beadDepth = gl.heightMm || 17;
            const inside = gl.position !== "buiten"; // default binnen (inside)
            const dir = inside ? 1 : -1;
            const glassFace = glassZOffset + dir * (thisGlassThickness / 2);
            const beadZ = glassFace + dir * (beadDepth / 2);
            addBorder(glassHostRect, beadWidth, beadDepth, beadMat, beadZ, "bead");
          }
        }
      }
    }

    // Dubbelzijdige divider-doorsnede (sponning aan beide vakzijden) voor het
    // layout-pad — het enkelzijdige kozijnprofiel wijst daar met zijn sponning
    // maar één kant op. Gesynthetiseerd op exacte dividerbreedte/kozijndiepte
    // (bibliotheek-tussenstijlen zijn breder dan het divider-rect).
    const synthSections = new Map();
    function layoutDividerSection(crossW) {
      const key = `divider|${baseMat}|${crossW.toFixed(1)}|${frameDepth.toFixed(1)}`;
      let s = synthSections.get(key);
      if (!s) {
        const woodish = baseMat === "wood" || baseMat === "wood_aluminum";
        const ledge = woodish ? Math.max(4, frameDepth - 51) : Math.min(25, frameDepth * 0.4);
        s = sectionFromPoints(key, dividerSectionContour(crossW, frameDepth, sponningHoogte, ledge));
        synthSections.set(key, s);
      }
      return s;
    }

    // Vertical dividers (tussenstijlen) — divider i zit tussen kolom i en
    // i+1; ofs-core bewaart zijn profiel op kolom i+1 (zie PropertiesPanel).
    // Op het layout-pad is het grid vestigiaal: daar geldt de divider-sectie.
    if (geometry.vDividers) {
      geometry.vDividers.forEach((rect, i) => {
        const section = layoutTree
          ? layoutDividerSection(rect.width)
          : getSection(kozijn.grid?.columns?.[i + 1]?.dividerProfile || kozijn.frame?.profile);
        const mesh = section && makeProfileMember(rect, section, dividerMat, "v");
        kozijnGroup.add(tag(mesh || makeBox(rect, frameDepth, dividerMat), "divider"));
      });
    }

    // Horizontal dividers (tussendorpels)
    if (geometry.hDividers) {
      geometry.hDividers.forEach((rect, i) => {
        const section = layoutTree
          ? layoutDividerSection(rect.height)
          : getSection(kozijn.grid?.rows?.[i + 1]?.dividerProfile || kozijn.frame?.profile);
        const mesh = section && makeProfileMember(rect, section, dividerMat, "h");
        kozijnGroup.add(tag(mesh || makeBox(rect, frameDepth, dividerMat), "divider"));
      });
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
    kozijnGroup.add(tag(wall, "wall"));

    // Opening cutout (simple: a dark box behind the glass, sized to the opening)
    const openingGeo = new THREE.BoxGeometry(preSize.x + 2, preSize.y + 2, wallDepth + 2);
    const openingMat = new THREE.MeshBasicMaterial({ color: 0x1a1a2e });
    const opening = new THREE.Mesh(openingGeo, openingMat);
    opening.position.set(preCenter.x, preCenter.y, -wallDepth / 2 - frameDepth / 2);
    kozijnGroup.add(tag(opening, "wall"));

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
  // Use JSON serialization to detect deep changes (Svelte 5 $effect tracks references only)
  let prevGeomJson = $state("");

  $effect(() => {
    const k = $currentKozijn;
    const g = $currentGeometry;
    const profiles = $allProfiles || [];
    if (!scene || !k || !g) return;
    // Per-cell signature so type / sash / glaslat / filling / glass changes rebuild the model
    const cellSig = (k.cells || []).map((c) => [
      c.panelType,
      c.sashWidth || "",
      c.glaslat ? `${c.glaslat.position}${c.glaslat.widthMm}${c.glaslat.heightMm}` : "",
      c.panelFilling ? `${c.panelFilling.fillingType}${c.panelFilling.thicknessMm}` : "",
      c.glazing?.thicknessMm || "",
    ].join(":")).join("|");
    // Profiel-signature: gekozen profiel-ids per lid + of er een echte
    // doorsnede voor beschikbaar is (rebuild zodra custom profielen laden).
    const f = k.frame || {};
    const usedRefs = [
      f.profile, f.sillProfile, f.topProfile, f.bottomProfile, f.leftProfile, f.rightProfile,
      ...(k.grid?.columns || []).map((c) => c?.dividerProfile),
      ...(k.grid?.rows || []).map((r) => r?.dividerProfile),
    ];
    const profSig = usedRefs.map((ref) => {
      if (!ref?.id) return "-";
      const def = profiles.find((p) => p.id === ref.id);
      return `${ref.id}:${Array.isArray(def?.crossSection) ? def.crossSection.length : 0}`;
    }).join(",");
    const newJson = JSON.stringify({
      id: k.id,
      w: f.outerWidth,
      h: f.outerHeight,
      cells: k.cells?.length,
      shape: f.shape?.shapeType,
      sig: cellSig,
      mat: getMaterialKey(f.material),
      fw: f.frameWidth,
      fd: f.frameDepth,
      prof: profSig,
      // Vleugel-/dividersecties komen ook uit de bibliotheek: rebuild zodra
      // die (async) binnenkomt, ook zonder expliciete profielrefs.
      plen: profiles.length,
      // Vrije indeling: compacte hash van de layout-boom (splits/vullingen).
      layout: k.layout ? hashStr(JSON.stringify(k.layout)) : "",
      // RAL-kleur voedt de materialen (getFrameColor); zonder dit veld verschijnt
      // een kleurwissel pas in 3D na een ongerelateerde bewerking.
      color: `${f.colorInside || ""}|${f.colorOutside || ""}`,
      // Hash van de geometrie-payload zelf — vangt layout-afgeleide vakken en
      // cellRect.vulling die pas na een async geometrie-refresh binnenkomen.
      geo: hashStr(JSON.stringify(g)),
    });
    if (newJson !== prevGeomJson) {
      prevGeomJson = newJson;
      build3DKozijn(scene, k, g, profiles);
    }
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
