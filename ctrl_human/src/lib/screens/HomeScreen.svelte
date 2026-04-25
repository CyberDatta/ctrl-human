<script lang="ts">
  import { tick, onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import '$lib/styles/tokens.css';
  import sittingImg from '$lib/assets/images/sitting_in_front_of_pc.png';
  import posingImg from '$lib/assets/images/posing_in_front_of_pc.png';
  import chunkyArrow from '$lib/assets/icons/chunky_arrow.svg';
  import errorIcon from '$lib/assets/icons/black_cross_red_background.svg';

  interface Webcam {
    index: number;
    name: string;
  }

  let schemeOpen = false;
  let webcamOpen = false;

  type Scheme = { controller_id: string; title: string };
  let schemes: Scheme[] = [];
  let webcams: Webcam[] = [];

  onMount(async () => {
    try {
      const controllers = await invoke<{ controller_id: string; title: string; hotkey: boolean }[]>('load_controllers');
      schemes = controllers.filter(c => c.hotkey).map(c => ({ controller_id: c.controller_id, title: c.title }));
    } catch (e) { console.error('load_controllers failed:', e); }
  });
  let webcamsLoading = false;

  let selectedScheme: Scheme | null = null;
  let selectedWebcam = 'Webcam';
  let showError = false;

  async function fetchWebcams() {
    webcamsLoading = true;
    try {
      webcams = await invoke<Webcam[]>('list_webcams');
    } catch (e) {
      console.error('Failed to list webcams:', e);
    } finally {
      webcamsLoading = false;
    }
  }

  function toggleScheme() {
    schemeOpen = !schemeOpen;
    webcamOpen = false;
  }

  async function toggleWebcam() {
    webcamOpen = !webcamOpen;
    schemeOpen = false;
    if (webcamOpen) {
      await tick();
      fetchWebcams();
    }
  }

  function pickScheme(scheme: Scheme) {
    selectedScheme = scheme;
    schemeOpen = false;
  }

  function truncate(text: string, max = 15): string {
    return text.length > max ? text.slice(0, max) + '…' : text;
  }

  function startPlaying() {
    if (!selectedScheme) {
      showError = true;
      return;
    }
    goto(
      `/controller-studio/controller-library/edit/test` +
      `?id=${selectedScheme.controller_id}` +
      `&name=${encodeURIComponent(selectedScheme.title)}` +
      `&back=${encodeURIComponent('/')}` +
      `&backText=${encodeURIComponent('Home')}`
    );
  }

  function dismissError() {
    showError = false;
  }

  async function pickWebcam(webcam: Webcam) {
    selectedWebcam = truncate(webcam.name);
    webcamOpen = false;
    try {
      await invoke('select_webcam', { index: webcam.index });
    } catch (e) {
      console.error('Failed to select webcam:', e);
    }
  }
</script>

<main class="home">
  <h1 class="title">Ctrl+Human</h1>

  <div class="row">
    <div class="box box-left">
      <img src={sittingImg} alt="Sitting in front of PC" class="hero-img" />
      <h2 class="box-heading">Controller Studio</h2>
      <p class="box-text">Create custom controllers by mapping your body movements to keyboard inputs</p>
      <button class="cta" on:click={() => goto('/controller-studio')}>Get Creative</button>
    </div>
    <div class="box box-right">
      <div class="dropdowns">
        <div class="dropdown">
          <button class="dropdown-toggle" class:active={schemeOpen} on:click={toggleScheme}>
            <span>{selectedScheme ? selectedScheme.title : 'Controller'}</span>
            <img src={chunkyArrow} alt="" class="dropdown-arrow" class:open={schemeOpen} />
          </button>
          {#if schemeOpen}
            <ul class="dropdown-menu">
              {#if schemes.length === 0}
                <li><span class="dropdown-item">No controllers enabled</span></li>
              {:else}
                {#each schemes as scheme}
                  <li><button class="dropdown-item" on:click={() => pickScheme(scheme)}>{scheme.title}</button></li>
                {/each}
              {/if}
            </ul>
          {/if}
        </div>
        <!-- <div class="dropdown">
          <button class="dropdown-toggle" class:active={webcamOpen} on:click={toggleWebcam}>
            <span>{selectedWebcam}</span>
            <img src={chunkyArrow} alt="" class="dropdown-arrow" class:open={webcamOpen} />
          </button>
          <ul class="dropdown-menu" style:display={webcamOpen ? '' : 'none'}>
            {#if webcamsLoading}
              <li><span class="dropdown-item">Loading...</span></li>
            {:else if webcams.length === 0}
              <li><span class="dropdown-item">No webcams found</span></li>
            {:else}
              {#each webcams as webcam}
                <li><button class="dropdown-item" on:click={() => pickWebcam(webcam)}>{truncate(webcam.name)}</button></li>
              {/each}
            {/if}
          </ul>
        </div> -->
      </div>
      <img src={posingImg} alt="Posing in front of PC" class="hero-img" />
      <h1 class="box-heading box-heading-2">Controller</h1>
      <p class="box-text box-text-2">Turn your body into the controller. All you need is a webcam.</p>
      <button class="cta cta-secondary" on:click={startPlaying}>Start Playing</button>
    </div>
  </div>

  {#if showError}
    <div class="overlay" on:click={dismissError}>
      <div class="error-modal" on:click|stopPropagation>
        <img src={errorIcon} alt="Error" class="error-icon" />
        <h2 class="error-title">Error!</h2>
        <p class="error-text">Please choose a controller scheme.</p>
        <button class="error-btn" on:click={dismissError}>Got It!</button>
      </div>
    </div>
  {/if}
</main>

<style>
 .home {
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  background-color: var(--color-secondary-2);
  padding-top: 20vh;
  padding-bottom: 5vh;
  position: relative;
}

.title {
  font-family: var(--font-primary);
  font-weight: var(--font-weight-Huge);
  font-size: var(--font-size-Huge);
  line-height: var(--line-height-Huge);
  font-style: var(--font-style-Huge);
  color: var(--color-dark-1);
  margin: 0;
  position: absolute;
  left: 6.5vw;
  top: 8.5vh;
}

  .row {
    display: flex;
    justify-content: space-between;
    margin-top: 1.5vh;
    padding: 0 5.5vw;
    flex: 1;
  }

  .box {
    width: 48%;
    border-radius: 0;
    padding-top: 5vh;
    box-sizing: border-box;
    padding-left: 7%;
    padding-right: 7%;
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-l);
  }

  .box-left {
    background-color: var(--color-background);
    display: flex;
    flex-direction: column;
  }

  .box-right {
    background-color: var(--color-secondary-3);
    display: flex;
    flex-direction: column;
    position: relative;
  }

  .hero-img {
    height: 28vh;
    border-radius: 0;
    object-fit: contain;
    padding-bottom: 2vh;
  }

  .box-heading {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-dark-1);
    margin: 1vh 0 0 0;
    padding-bottom: 1vh;
  }

  .box-heading-2 {
    color: var(--color-white);
  }

  .box-text {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 0.5vh 0 0 0;
    padding-bottom: 2.5vh;
  }

  .box-text-2 {
    color: var(--color-white);
  }

  .cta {
    margin-top: 1vh;
    background-color: var(--color-primary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-radius: 0;
    padding: 0.75rem 1.5rem;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    align-self: flex-start;
    box-shadow: var(--shadow-m);
  }

  .cta:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .cta-secondary {
    background-color: var(--color-secondary-1);
  }

  /* Dropdowns — float above the box content */
  .dropdowns {
    position: absolute;
    top: 5vh;
    left: 7%;
    right: 7%;
    display: flex;
    gap: 1rem;
    z-index: 10;
  }

  .dropdown {
    position: relative;
  }

  .dropdown-toggle {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1.125rem 1rem;
    background-color: var(--color-primary-3);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-radius: 0;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    box-shadow: var(--shadow-m);
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
    width: 2rem;
    height: 2rem;
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
    background-color: var(--color-primary-3);
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
    padding: 1.125rem 1rem;
    background: none;
    border: none;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    text-align: left;
    box-sizing: border-box;
  }

  .dropdown-item:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }

  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: var(--color-dark-2);
    z-index: 100;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .error-modal {
    width: 31vw;
    height: 47vh;
    background-color: var(--color-secondary-4);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    box-shadow: var(--shadow-l);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding-top: 4.5vh;
    padding-bottom: 5.5vh;
    padding-left: 2vw;
    padding-right: 2vw;
    box-sizing: border-box;
  }

  .error-icon {
    width: 5rem;
    height: 5rem;
    padding-bottom: 2vh;
  }

  .error-title {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H2);
    font-size: var(--font-size-H2);
    line-height: var(--line-height-H2);
    color: var(--color-dark-1);
    margin: 0;
  }

  .error-text {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H4);
    font-size: var(--font-size-H4);
    line-height: var(--line-height-H4);
    color: var(--color-dark-1);
    margin: 1vh 0 2vh 0;
    text-align: left;
  }

  .error-btn {
    background-color: var(--color-secondary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-radius: 0;
    padding-top: 0.875rem;
    padding-bottom: 0.875rem;
    padding-left: 7vw;
    padding-right: 7vw;
    cursor: pointer;
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H3);
    font-size: var(--font-size-H3);
    line-height: var(--line-height-H3);
    color: var(--color-dark-1);
    box-shadow: var(--shadow-m);
  }

  .error-btn:hover {
    background-color: var(--color-mouse-hover);
    color: var(--color-white);
  }
</style>
