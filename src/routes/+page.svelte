<script>
  import { fade } from 'svelte/transition';
  let page = $state(0);
  let showMIT = $state(false);
  let showApache = $state(false);
  import Apache from './apache.svelte';
  import Mit from './mit.svelte';
</script>

<main class="p-strip">
  <div class="row">
    <div class="col-8 col-start-3">
      {#if page == 0}
        <div in:fade="{{ duration: 400 }}">
          <!-- 标题行 -->
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem;">
            <h1 class="p-heading--1">欢迎来到 musicdownloader!</h1>
            <button class="p-button--link" onclick={() => { page = 1; }}>关于</button>
          </div>

          <!-- 卡片保留 padding -->
          <div class="p-card--highlighted" style="padding: 2rem;">
            <h2 class="p-heading--2">gequhai.com</h2>
            <h3 class="p-heading--3">用法：</h3>
            <p>搜索歌名或歌手，点击结果，你的下载将自动开始。</p>
            <p>音频文件（mp3、aac）将自动存到系统Music文件夹的musicdownloaded子文件夹里</p>
            <a class="p-button--positive" href="https://www.gequhai.com">开始</a>
          </div>
        </div>

      {:else if page == 1}
        <div in:fade="{{ duration: 400 }}">
          <!-- 返回按钮 -->
          <div style="margin-bottom: 1rem;">
            <button class="p-button" onclick={() => { page = 0; }}>← 返回</button>
          </div>

          <!-- 卡片保留 padding -->
          <div class="p-card--highlighted" style="padding: 2rem;">
            <p>musicdownloader 是一个音乐下载器，旨在用最少的操作下载音乐。</p>
            <p>本软件使用的API均由第三方免费提供。不保障可用性。请不要滥用。</p>

            <hr class="p-divider" style="margin: 1.5rem 0;" />
            <p>本软件是开源软件。使用MIT或Apache 2.0开源协议。你可以<a class="p-button--link" href="https://github.com/abpdf/musicdownloader" target="_blank">在此处</a>查看源代码。</p>

            <!-- MIT -->
            <div style="margin-bottom: 0.5rem;">
              <div
                style="cursor: pointer; user-select: none; display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0;"
                onclick={() => { showMIT = !showMIT; }}
              >
                <strong>MIT License</strong>
                <span>{showMIT ? '▾' : '▸'}</span>
              </div>
              {#if showMIT}
                <div in:fade="{{ duration: 400 }}" style="padding-left: 1rem; color: #666;">
                  <p><Mit /></p>
                </div>
              {/if}
            </div>

            <!-- Apache -->
            <div>
              <div
                style="cursor: pointer; user-select: none; display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0;"
                onclick={() => { showApache = !showApache; }}
              >
                <strong>Apache License 2.0</strong>
                <span>{showApache ? '▾' : '▸'}</span>
              </div>
              {#if showApache}
                <div in:fade="{{ duration: 400 }}" style="padding-left: 1rem; color: #666;">
                  <p><Apache /></p>
                </div>
              {/if}
            </div>
          </div>
        </div>
      {/if}
    </div>
  </div>
</main>