<script>
  import { _ } from "svelte-i18n";
  import { currentGeometry, currentKozijn, updateCornerJoints } from "../../stores/kozijn.js";
  import { archJointLabel, ringSeamCorners } from "../../lib/dividerJoinery.js";

  let { visible = true } = $props();
  let shape = $derived($currentKozijn?.frame.shape?.shapeType);
  let round = $derived(shape === "round");
  let arched = $derived(shape === "arched" || shape === "arched_trapezoid");

  const CORNER_LABELS = ["Linksboven", "Rechtsboven", "Linksonder", "Rechtsonder"];

  const JOINT_TYPES = [
    { value: "pen_slis", label: "Pen/slis" },
    { value: "verstek", label: "Verstek (45°)" },
    { value: "contramal", label: "Contramal" },
    { value: "stomp", label: "Stomp" },
  ];

  const THROUGH_TYPES = [
    { value: "stijl", label: "Stijl (verticaal)" },
    { value: "dorpel", label: "Dorpel (horizontaal)" },
  ];

  let indexes = $derived.by(() => {
    if (!round) return [0, 1, 2, 3];
    const frame = $currentKozijn.frame;
    const timber = m => m?.wood != null || m === "wood_aluminum";
    if (![frame.topMaterial ?? frame.material, frame.bottomMaterial ?? frame.material].every(timber)) return [];
    return ringSeamCorners($currentGeometry);
  });

  function getJoints() {
    const k = $currentKozijn;
    if (k?.frame.jointsConfigured && k.frame.cornerJoints?.length >= 4) return k.frame.cornerJoints;
    if ($currentGeometry?.cornerSettings) return $currentGeometry.cornerSettings;
    if (!k || !k.frame.cornerJoints || k.frame.cornerJoints.length < 4) {
      // Default: 4 pen/slis joints
      return Array(4).fill(null).map(() => ({
        jointType: "pen_slis",
        throughMember: "stijl",
        angle: 90,
        penLength: 20,
      }));
    }
    return k.frame.cornerJoints;
  }

  function typeLabel(type, archFoot) {
    if (type.value === "pen_slis" && (round || archFoot)) return $_("archJoint.penAndMortise");
    if (round && type.value === "stomp") return $_("ringJoint.butt");
    if (arched && type.value === "verstek") return $_("member.miter");
    return type.label;
  }

  function throughLabel(type, archFoot) {
    if (round) return $_(type.value === "dorpel" ? "member.lowerArc" : "member.upperArc");
    if (archFoot) return $_(type.value === "dorpel" ? "archJoint.throughArc" : "archJoint.throughStile");
    return type.label;
  }

  async function updateJoint(index, field, value) {
    const joints = [...getJoints()];
    joints[index] = { ...joints[index], [field]: value };

    // Auto-set angle based on joint type
    if (field === "jointType") {
      joints[index].angle = value === "verstek" ? 45 : 90;
      joints[index].penLength = value === "pen_slis" ? 20 : 0;
    }

    await updateCornerJoints(joints);
  }

  function editPenLength(index, input, previous) {
    if (Number.isFinite(input.valueAsNumber) && input.validity.valid) {
      updateJoint(index, "penLength", input.valueAsNumber);
    } else {
      input.value = previous;
    }
  }

  /** Apply same joint type to all 4 corners at once */
  async function applyToAll(jointType) {
    const joints = getJoints().map(j => ({
      ...j,
      jointType,
      angle: jointType === "verstek" ? 45 : 90,
      penLength: jointType === "pen_slis" ? 20 : 0,
    }));
    await updateCornerJoints(joints);
  }
</script>

{#if visible && $currentKozijn}
  <div class="joint-panel">
    <h3>{round ? $_("ringJoint.title") : "Hoekverbindingen"}</h3>
    {#if round}
      <p class="joint-note">{$_("member.radialJoints")}</p>
    {:else}
    {#if arched}<p class="joint-note">{$_("member.archConnections")}</p>{/if}
    <!-- Quick apply buttons -->
    <div class="quick-apply">
      <span class="label">Alle hoeken:</span>
      <button class="chip" onclick={() => applyToAll("pen_slis")}>Pen/slis</button>
      <button class="chip" onclick={() => applyToAll("verstek")}>Verstek</button>
      <button class="chip" onclick={() => applyToAll("stomp")}>Stomp</button>
    </div>
    {/if}

    {#each indexes as i (i)}
      {@const joint = getJoints()[i]}
      {@const archFoot = arched && i < 2}
      <div class="corner-config">
        <div class="corner-header">
          <span class="corner-indicator" style="background: {joint.jointType === 'pen_slis' ? '#22c55e' : joint.jointType === 'verstek' ? '#3b82f6' : joint.jointType === 'contramal' ? '#f59e0b' : '#ef4444'}"></span>
          <span class="corner-name">{round || archFoot ? archJointLabel($_, i, round ? "ring_seam" : "springing") : CORNER_LABELS[i]}</span>
        </div>
        <div class="corner-fields">
          <select
            aria-label={$_("archJoint.type")}
            value={round && joint.jointType !== "pen_slis" ? "stomp" : joint.jointType}
            onchange={(e) => updateJoint(i, "jointType", e.target.value)}
          >
            {#each (round ? JOINT_TYPES.filter(t => t.value === "pen_slis" || t.value === "stomp") : JOINT_TYPES) as jt}
              <option value={jt.value}>{typeLabel(jt, archFoot)}</option>
            {/each}
          </select>
          {#if !round || joint.jointType === "pen_slis"}
          {#if round}<label for={`ring-donor-${i}`}>{$_("archJoint.tenonOn")}</label>{/if}
          <select id={round ? `ring-donor-${i}` : undefined}
            aria-label={$_(round ? "archJoint.tenonOn" : "archJoint.through")}
            value={joint.throughMember}
            onchange={(e) => updateJoint(i, "throughMember", e.target.value)}
          >
            {#each THROUGH_TYPES as tt}
              <option value={tt.value}>{throughLabel(tt, archFoot)}</option>
            {/each}
          </select>
          {/if}
          {#if joint.jointType === "pen_slis"}
            <div class="pen-length">
              <label for={`joint-length-${i}`}>{$_("joinery.tenonLength")}</label>
              <input
                id={`joint-length-${i}`}
                type="number"
                value={joint.penLength}
                onchange={(e) => editPenLength(i, e.target, joint.penLength)}
                min="10" max="50" step="any"
              />
              <span class="unit">mm</span>
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .joint-panel {
    padding: var(--sp-3);
  }

  .joint-note { font-size: 12px; line-height: 1.4; color: var(--text-secondary); }

  h3 {
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 var(--sp-3);
  }

  .quick-apply {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-3);
    flex-wrap: wrap;
  }

  .quick-apply .label {
    font-size: 11px;
    color: var(--text-muted);
  }

  .chip {
    padding: 2px 8px;
    font-size: 10px;
    font-weight: 600;
    border: 1px solid var(--border-color, #444);
    border-radius: 10px;
    background: transparent;
    color: var(--text-secondary);
    cursor: default;
    transition: all 0.15s;
  }

  .chip:hover {
    background: var(--amber);
    color: var(--bg-surface);
    border-color: var(--amber);
  }

  .corner-config {
    margin-bottom: var(--sp-3);
    padding: var(--sp-2);
    background: var(--bg-surface-alt);
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-color, #333);
  }

  .corner-header {
    display: flex;
    align-items: center;
    gap: var(--sp-2);
    margin-bottom: var(--sp-2);
  }

  .corner-indicator {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .corner-name {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-primary);
  }

  .corner-fields {
    display: flex;
    flex-direction: column;
    gap: var(--sp-1);
  }

  .corner-fields select {
    width: 100%;
    padding: 3px 6px;
    font-size: 11px;
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-color, #444);
    border-radius: var(--radius-sm);
  }

  .pen-length {
    display: flex;
    align-items: center;
    gap: var(--sp-1);
    font-size: 11px;
    color: var(--text-secondary);
  }

  .pen-length input {
    width: 50px;
    padding: 2px 4px;
    font-size: 11px;
    background: var(--bg-surface);
    color: var(--text-primary);
    border: 1px solid var(--border-color, #444);
    border-radius: var(--radius-sm);
    text-align: center;
  }

  .unit {
    color: var(--text-muted);
    font-size: 10px;
  }
</style>
