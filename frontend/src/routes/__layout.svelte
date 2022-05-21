<script context="module">
  import { locale, loadTranslations } from '$lib/i18n';

  export async function load({ url }) {
    const { pathname } = url;

    const defaultLocale = 'en'; // TODO: get from cookie, user session, ...
    const initLocale = locale.get() || defaultLocale;

    await loadTranslations(initLocale, pathname);

    return {};
  }
</script>

<script>
  import { t as _ } from "$lib/i18n";
  import "../global.scss";
  import { page } from "$app/stores";
</script>

<header>
  <div class="flex items-center">
    <div class="flex items-center mr-auto">
      <a href="/">
        <img src="/logo.svg" style="height: 30px;" class="inline-block mr-2" />
        <span class="font-bold">{$_("site_name")}</span>
      </a>
    </div>

    <nav>
      <a href="/" class:active={$page.url.pathname === "/"}>{$_("nav.dashboard")}</a>
      <a href="/block" class:active={$page.url.pathname.startsWith("/block")}>{$_("nav.blocks")}</a>
      <a href="/tx" class:active={$page.url.pathname.startsWith("/tx")}>{$_("nav.txns")}</a>
    </nav>
  </div>
</header>

<main>
  <slot />
</main>

<style lang="scss">
  header {
    @apply p-5 bg-transparent border-b border-slate-50/[0.06] text-white;

    a {
      @apply text-white;
    }

    & > div {
      max-width: 90rem;
      @apply mx-auto;
    }

    nav {
      @apply space-x-5;

      a {
        &:hover {
          @apply text-slate-400;
        }

        &.active {
          @apply text-orange-400;
        }
      }
    }
  }

  main {
    @apply mx-5;
  }
</style>
