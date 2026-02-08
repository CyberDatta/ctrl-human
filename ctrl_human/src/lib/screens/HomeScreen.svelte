<script lang="ts">
  import '$lib/styles/tokens.css';
  import sittingImg from '$lib/assets/images/sitting_in_front_of_pc.png';
  import posingImg from '$lib/assets/images/posing_in_front_of_pc.png';
</script>

<main class="home">
  <h1 class="title">Ctrl+Human</h1>

  <div class="row">
    <div class="box box-left">
      <img src={sittingImg} alt="Sitting in front of PC" class="hero-img" />
      <h2 class="box-heading">Controller Studio</h2>
      <p class="box-text">Create custom control schemes by mapping your body movements to keyboard inputs</p>
      <button class="cta">Get Creative</button>
    </div>
    <div class="box box-right">
      <img src={posingImg} alt="Posing in front of PC" class="hero-img" />
      <h1 class="box-heading box-heading-2">Controller</h1>
      <p class="box-text box-text-2">Turn your body into the controller. All you need is a webcam.</p>
      <button class="cta cta-secondary">Start Playing</button>
    </div>
  </div>
</main>

<style>
 .home {
  width: 100%;
  height: 100vh;
  padding: 0rem;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  background-color: var(--color-secondary-2);

  /* Push content down so it starts below the absolutely-positioned title */
  padding-top: 14rem;
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
  left: 6.0625rem;
  top: 5.125rem;
}

  .row {
    display: flex;
    justify-content: space-between;
    margin-top: 1.5rem;
    padding: 0 5rem;
    flex: 1;
  }

  .box {
    width: 52rem;
    height: 49.3125rem;
    border-radius: 0rem;
    padding-top: 4.1875rem;
    box-sizing: border-box;
    padding-left: 4rem;
    padding-right: 4rem;
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
  }

  .hero-img {
    height: 21.4375rem;
    border-radius: 0rem;
    object-fit: contain;
    padding-bottom: 2.5rem;
  }

  .box-heading {
    font-family: var(--font-primary);
    font-weight: var(--font-weight-H1);
    font-size: var(--font-size-H1);
    line-height: var(--line-height-H1);
    color: var(--color-dark-1);
    margin: 1rem 0 0 0;
    padding-bottom: 1rem;
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
    margin: 0.5rem 0 0 0;
    padding-bottom: 2rem;
  }

    .box-text-2 {
    color: var(--color-white);
    }

  .cta {
    margin-top: 1rem;
    background-color: var(--color-primary-2);
    border: var(--stroke-width-s) solid var(--color-dark-1);
    border-radius: 0rem;
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
</style>
