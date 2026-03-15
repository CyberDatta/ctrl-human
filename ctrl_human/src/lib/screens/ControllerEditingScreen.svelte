<script lang="ts">
  import { goto } from '$app/navigation';
  import '$lib/styles/tokens.css';
  import pencilIcon from '$lib/assets/icons/pencil.svg';
  import labFlaskIcon from '$lib/assets/icons/lab_flask.svg';
  import chunkyArrow from '$lib/assets/icons/chunky_arrow.svg';

  function goBack() {
    goto('/controller-studio/controller-library');
  }

  // ── Title editing ──
  let schemeName = 'Untitled Scheme 5';
  let editingTitle = false;
  let titleDraft = '';
  function startEditTitle() { titleDraft = schemeName; editingTitle = true; }
  function commitTitle() { if (titleDraft.trim()) schemeName = titleDraft.trim(); editingTitle = false; }
  function onTitleKey(e: KeyboardEvent) {
    if (e.key === 'Enter') commitTitle();
    if (e.key === 'Escape') editingTitle = false;
  }

  // ── Description editing ──
  let description = 'This is a sample description of what this project does. This is a sample description of what this project does. This is a sample description of what this project does.';
  let editingDesc = false;
  let descDraft = '';
  function startEditDesc() { descDraft = description; editingDesc = true; }
  function commitDesc() { if (descDraft.trim()) description = descDraft.trim(); editingDesc = false; }

  // ── Dropdown options ──
  const priorityOptions = ['1 - Highest', '2', '3', '4', '5 - Lowest'];
  const inputTypeOptions = ['Keyboard', 'Mouse Button', 'Mouse Move', 'Gamepad'];

  // ── Per-row dropdown state ──
  type DropdownField = 'priority' | 'inputType';
  let openDropdown: { rowId: number; field: DropdownField } | null = null;

  function toggleDropdown(rowId: number, field: DropdownField) {
    if (openDropdown?.rowId === rowId && openDropdown?.field === field) {
      openDropdown = null;
    } else {
      openDropdown = { rowId, field };
    }
  }

  function pickOption(rowId: number, field: DropdownField, value: string) {
    poseRows = poseRows.map(r => r.id === rowId ? { ...r, [field]: value } : r);
    openDropdown = null;
  }

  // ── Pose mappings ──
  type PoseRow = { id: number; poseName: string; priority: string; inputType: string; input: string };
  let poseRows: PoseRow[] = [
    { id: 1, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 2, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 3, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 4, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 5, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 6, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 7, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 8, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
    { id: 9, poseName: 'Wave 2 hand', priority: 'Priority', inputType: 'Input Type', input: 'Input' },
  ];
  let nextId = Math.max(...poseRows.map(r => r.id)) + 1;

  function addPose(poseName: string) {
    poseRows = [...poseRows, { id: nextId++, poseName, priority: 'Priority', inputType: 'Input Type', input: 'Input' }];
    openDropdown = null;
  }

  function removeRow(id: number) {
    poseRows = poseRows.filter(r => r.id !== id);
    openDropdown = null;
  }

  // ── Available poses (sidebar) ──
  const availablePoses = [
    { name: 'Wave one hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
    { name: 'Wave 2 hand' },
  ];
</script>

<div class="controller-editor">

  <!-- ── Main panel ── -->
  <div class="main-panel">

    <!-- Nav -->
    <nav class="top-bar">
      <button class="back-btn" on:click={goBack}>
        <svg class="back-arrow" viewBox="0 0 33 40" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M11,5 L21,5 L21,19 L28,19 L26.083,23.923 L24.167,23.923 L20.333,31.308 L18.554,31.308 L17.778,35 L15.222,35 L14.583,31.308 L12.667,31.308 L9.472,23.923 L6.917,23.923 L5,19 L11,19 Z" />
        </svg>
        <span class="back-text">Controller Library</span>
      </button>
    </nav>

    <!-- Title -->
    <div class="title-row">
      {#if editingTitle}
        <input
          class="title-input"
          bind:value={titleDraft}
          on:blur={commitTitle}
          on:keydown={onTitleKey}
          autofocus
        />
      {:else}
        <span class="title-wrap">
          <h1 class="scheme-title">{schemeName}</h1>
          <button class="pencil-btn" on:click={startEditTitle} aria-label="Edit title">
            <img src={pencilIcon} alt="" class="pencil-icon" />
          </button>
        </span>
      {/if}
    </div>

    <!-- Description -->
    <div class="desc-row">
      {#if editingDesc}
        <textarea
          class="desc-input"
          bind:value={descDraft}
          on:blur={commitDesc}
          autofocus
        ></textarea>
      {:else}
        <p class="scheme-desc">{description}</p>
        <button class="pencil-btn pencil-btn-desc" on:click={startEditDesc} aria-label="Edit description">
          <img src={pencilIcon} alt="" class="pencil-icon" />
        </button>
      {/if}
    </div>

    <!-- Action buttons -->
    <div class="action-row">
      <button class="action-btn test-btn">
        <!-- <img src={labFlaskIcon} alt="" class="action-icon" /> -->
        Test Scheme
      </button>
      <button class="action-btn export-btn">Export Scheme</button>
      <button class="action-btn delete-btn">Delete Scheme</button>
    </div>

    <!-- Detection Method card -->
    <div class="detection-card">
      <h2 class="detection-heading">Controller Setup</h2>
      <div class="pose-rows">
        {#each poseRows as row (row.id)}
          <div class="pose-row">
            <span class="pose-row-name">{row.poseName}</span>
            <div class="pose-row-controls">

              <!-- Priority dropdown -->
              <div class="dropdown">
                <button
                  class="dropdown-toggle"
                  class:active={openDropdown?.rowId === row.id && openDropdown?.field === 'priority'}
                  on:click={() => toggleDropdown(row.id, 'priority')}
                >
                  <span>{row.priority}</span>
                  <img src={chunkyArrow} alt="" class="dropdown-arrow" class:open={openDropdown?.rowId === row.id && openDropdown?.field === 'priority'} />
                </button>
                {#if openDropdown?.rowId === row.id && openDropdown?.field === 'priority'}
                  <ul class="dropdown-menu">
                    {#each priorityOptions as opt}
                      <li>
                        <button class="dropdown-item" on:click={() => pickOption(row.id, 'priority', opt)}>
                          {opt}
                        </button>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>

              <!-- Input Type dropdown -->
              <div class="dropdown">
                <button
                  class="dropdown-toggle"
                  class:active={openDropdown?.rowId === row.id && openDropdown?.field === 'inputType'}
                  on:click={() => toggleDropdown(row.id, 'inputType')}
                >
                  <span>{row.inputType}</span>
                  <img src={chunkyArrow} alt="" class="dropdown-arrow" class:open={openDropdown?.rowId === row.id && openDropdown?.field === 'inputType'} />
                </button>
                {#if openDropdown?.rowId === row.id && openDropdown?.field === 'inputType'}
                  <ul class="dropdown-menu">
                    {#each inputTypeOptions as opt}
                      <li>
                        <button class="dropdown-item" on:click={() => pickOption(row.id, 'inputType', opt)}>
                          {opt}
                        </button>
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>

              <!-- Input (plain button for now) -->
              <button class="input-btn">{row.input}</button>

              <!-- Remove -->
              <button class="remove-btn" on:click={() => removeRow(row.id)} aria-label="Remove pose">
                <svg class="remove-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="2"/>
                  <line x1="8" y1="8" x2="16" y2="16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                  <line x1="16" y1="8" x2="8" y2="16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                </svg>
              </button>

            </div>
          </div>
        {/each}
      </div>
    </div>

  </div>

  <!-- ── Right sidebar ── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2 class="sidebar-title">Available Poses</h2>
      <p class="sidebar-subtitle">Click the pose you wish to add to your controller scheme. You can attach multiple inputs to a single pose.</p>
    </div>
    <div class="sidebar-scroll">
      {#each availablePoses as pose}
        <button class="pose-card" on:click={() => addPose(pose.name)}>
          {pose.name}
        </button>
      {/each}
    </div>
  </aside>

</div>

<style>
  /* ── Root layout ── */
  .controller-editor {
    display: flex;
    flex-wrap: nowrap;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-tertiary-3);
    font-family: var(--font-primary);
  }

  /* ── Main panel ── */
  .main-panel {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 3rem 3.5rem 4rem 3.5rem;
    display: flex;
    flex-direction: column;
  }

  /* ── Nav ── */
  .top-bar {
    padding-bottom: 2.5rem;
  }

  .back-btn {
    display: flex;
    align-items: center;
    gap: 2rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .back-arrow {
    height: 4rem;
    transform: rotate(90deg);
    flex-shrink: 0;
  }

  .back-arrow path {
    fill: var(--color-background);
    stroke: var(--color-primary-outline);
    stroke-width: var(--stroke-width-xs);
    vector-effect: non-scaling-stroke;
  }

  .back-text {
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-background);
    -webkit-text-stroke: var(--stroke-width-xs) var(--color-primary-outline);
  }

  /* ── Title ── */
  .title-row {
    margin-bottom: 1.5rem;
  }

  .title-wrap {
    display: inline-block;
    position: relative;
  }

  .scheme-title {
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    color: var(--color-dark-1);
    margin: 0;
  }

  .title-input {
    font-weight: var(--font-weight-Huge);
    font-size: var(--font-size-Huge);
    line-height: var(--line-height-Huge);
    font-style: var(--font-style-Huge);
    font-family: var(--font-primary);
    color: var(--color-dark-1);
    background: rgba(255, 255, 255, 0.5);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    outline: none;
    padding: 0.25rem 0.75rem;
    max-width: 40rem;
  }

  /* ── Description ── */
  .desc-row {
    display: flex;
    align-items: flex-start;
    gap: 1.25rem;
    margin-bottom: 2.5rem;
    max-width: 45rem;
  }

  .scheme-desc {
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 0;
    flex: 1;
  }

  .desc-input {
    font-weight: 700;
    font-size: var(--font-size-H5);
    line-height: 1.9rem;
    font-family: var(--font-primary);
    color: var(--color-dark-1);
    background: rgba(255, 255, 255, 0.5);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    outline: none;
    padding: 0.5rem 0.75rem;
    resize: vertical;
    min-height: 6rem;
    flex: 1;
    width: 100%;
  }

  /* ── Pencil buttons ── */
  .pencil-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    flex-shrink: 0;
    position: absolute;
    top: -0.5rem;
    right: -2.75rem;
  }

  .pencil-btn-desc {
    position: static;
    margin-top: 0;
  }

  .pencil-icon {
    width: 2rem;
    height: 2rem;
    opacity: 0.85;
  }

  .pencil-btn:hover .pencil-icon {
    opacity: 1;
  }

  /* ── Action buttons row ── */
  .action-row {
    display: flex;
    gap: 1.25rem;
    margin-bottom: 2.5rem;
    align-items: center;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.6rem;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    padding: 0.75rem 1.5rem;
  }

  .action-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .action-icon {
    width: 2rem;
    height: 2rem;
    flex-shrink: 0;
  }

  .test-btn  { background-color: var(--color-primary-4); }
  .export-btn { background-color: var(--color-tertiary-2); }
  .delete-btn { background-color: var(--color-window); }

  /* ── Detection Method card ── */
  .detection-card {
    width: 60.375rem;
    background-color: var(--color-secondary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-l);
    padding: 2rem 2.5rem 2.5rem 2.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .detection-heading {
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-dark-1);
    margin: 0;
  }

  .pose-rows {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  /* ── Pose row ── */
  .pose-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background-color: var(--color-secondary-4);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-s);
    padding: 0.85rem 1.25rem;
    gap: 1.5rem;
  }

  .pose-row-name {
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    flex-shrink: 0;
  }

  .pose-row-controls {
    display: flex;
    align-items: center;
    gap: 2rem;
  }

  .pose-row-controls .input-btn {
    margin-right: 2rem;
  }

  /* ── Dropdown ── */
  .dropdown {
    position: relative;
  }

  .dropdown-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background-color: var(--color-primary-1);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-s);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-dark-1);
    padding: 0.5rem 0.85rem;
    white-space: nowrap;
  }

  .dropdown-toggle:hover,
  .dropdown-toggle.active {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .dropdown-toggle:hover .dropdown-arrow,
  .dropdown-toggle.active .dropdown-arrow {
    filter: brightness(0) invert(1);
  }

  .dropdown-arrow {
    width: 1.5rem;
    height: 1.5rem;
    flex-shrink: 0;
    transition: transform 0.2s ease;
  }

  .dropdown-arrow.open {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: 100%;
    left: 0;
    min-width: 100%;
    margin: 0;
    padding: 0;
    list-style: none;
    background-color: var(--color-primary-1);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-top: none;
    box-shadow: var(--shadow-m);
    z-index: 20;
  }

  .dropdown-menu li {
    border-bottom: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .dropdown-menu li:last-child {
    border-bottom: none;
  }

  .dropdown-item {
    display: block;
    width: 100%;
    padding: 0.6rem 0.85rem;
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-dark-1);
    text-align: left;
    box-sizing: border-box;
    white-space: nowrap;
  }

  .dropdown-item:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  /* ── Input plain button ── */
  .input-btn {
    background-color: var(--color-primary-1);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-s);
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H5);
    font-size: var(--font-size-H5);
    line-height: var(--line-height-H5);
    color: var(--color-dark-1);
    padding: 0.5rem 0.85rem;
    white-space: nowrap;
  }

  .input-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  /* ── Remove button ── */
  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-primary-1);
    flex-shrink: 0;
  }

  .remove-btn:hover {
    color: var(--color-secondary-1);
  }

  .remove-icon {
    width: 2.25rem;
    height: 2.25rem;
  }

  /* ── Right sidebar ── */
  .sidebar {
    width: 33.125rem;
    flex-shrink: 0;
    height: 100vh;
    display: flex;
    flex-direction: column;
    border-left: var(--stroke-width-s) solid var(--color-dark-1);
  }

  .sidebar-header {
    background-color: var(--color-secondary-4);
    padding: 2rem 2rem 1.75rem 2rem;
    border-bottom: var(--stroke-width-s) solid var(--color-dark-1);
    flex-shrink: 0;
  }

  .sidebar-title {
    font-weight: var(--font-weight-H2);
    font-size: var(--font-size-H2);
    line-height: var(--line-height-H2);
    color: var(--color-dark-1);
    margin: 0 0 0.75rem 0;
  }

  .sidebar-subtitle {
    font-size: var(--font-size-Body-L);
    line-height: var(--line-height-Body-L);
    color: var(--color-dark-1);
    margin: 0;
  }

  .sidebar-scroll {
    flex: 1;
    overflow-y: auto;
    background-color: var(--color-primary-3);
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* ── Sidebar pose cards ── */
  .pose-card {
    background-color: var(--color-background);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-m);
    padding: 1.5rem 1.5rem;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    text-align: left;
  }

  .pose-card:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }
</style>
