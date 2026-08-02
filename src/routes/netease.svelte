<script>
// ============ 方案二：独立键名（每个ID单独存） ============
const FlagDB2 = {
  // 生成唯一键名（统一加前缀，方便管理）
  _getKey(id) {
    return 'flag_' + id; // 例如：flag_1001
  },

  // 1. 存/改：设置某个ID的状态
  set(id, value) {
    localStorage.setItem(this._getKey(id), JSON.stringify(value));
  },

  // 2. 查：获取某个ID的状态
  //    未设置返回 undefined，设置过返回 true/false
  get(id) {
    const val = localStorage.getItem(this._getKey(id));
    return val !== null ? JSON.parse(val) : undefined;
  },

  // 3. 删：删除某个ID的记录
  delete(id) {
    localStorage.removeItem(this._getKey(id));
  },

  // 4. 清空：删除所有 flag_ 开头的键（批量清空）
  clear() {
    const keysToRemove = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key && key.startsWith('flag_')) {
        keysToRemove.push(key);
      }
    }
    keysToRemove.forEach(key => localStorage.removeItem(key));
  }
};




    import { invoke } from "@tauri-apps/api/core";
    let result = $state([]);
    let name = $state();
    let limit = $state(30);

    let queue = [];

    async function search() {
        result = [];
        const url = `https://api.qijieya.cn/meting/?type=search&id=${encodeURIComponent(name)}&limit=${limit}&server=netease`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`HTTP 错误! 状态码: ${response.status}`);
            }
            result = await response.json();
        } catch (error) {
            console.error("搜索失败：", error);
        }
    }
    function sanitizeFileName(name) {
        if (typeof name !== "string") return "";

        // 第一步：将 / 和 \ 全部替换为 &（and 符号）
        let result = name.replace(/[\\\/]/g, "&");

        // 第二步：移除其他非法字符（如 : * ? " < > |）
        // 注意：& 已被保留，不会在这里被移除
        result = result.replace(/[:*?"<>|]/g, "");

        // 第三步：去除首尾空格（Windows 不允许文件名以空格结尾）
        return result.trim();
    }
    function getId(url){
        const parsed = new URL(url);
        return parsed.searchParams.get('id');
    }
    async function getFinalMp3Url(apiUrl) {
        const response = await fetch(apiUrl, { method: "HEAD" });
        // response.url 此时就是最后的 MP3 直链，没有下载任何文件内容
        return response.url;
    }

    function addQueue(a) {
        if (queue.length === 0) {
            queue.push(a);
            doWork();
        } else {
            a.status = "waiting";
            queue.push(a);
        }
    }

    function downloadAll() {
        for (let i = 0; i < result.length; i++) {
            if (
                result[i].status != "Done" &&
                result[i].status != "waiting" &&
                result[i].status != "downloading"&&
                FlagDB2.get(getId(result[i].url)) !== true
            ) {
                addQueue(result[i]);
            }
        }
    }

    async function doWork() {
        while (queue.length > 0) {
            try {
                await download(queue[0]);
            } catch (error) {}
            queue.shift();
        }
    }

    async function download(u) {
        u.status = "downloading";
        const finalurl = await getFinalMp3Url(u.url);
        const status = await invoke("download_file_async", {
            url: finalurl,
            name: sanitizeFileName(u.name + "-" + u.artist),
        });
        if (status==="Done"){
            FlagDB2.set(getId(u.url),true);
        }
        u.status = status;
    }
</script>

<div class="p-card--highlighted" style="padding: 2rem;">
    <form class="p-search-box" on:submit|preventDefault={() => {}}>
        <label class="u-off-screen" for="search">Search</label>
        <input
            type="search"
            id="search"
            class="p-search-box__input"
            placeholder="点击输入…"
            bind:value={name}
        />
        <button type="reset" class="p-search-box__reset">
            <i class="p-icon--close">Close</i>
        </button>
        <button type="submit" class="p-search-box__button" on:click={search}>
            <i class="p-icon--search">Search</i>
        </button>
    </form>
    <div class="p-slider__wrapper">
    <div style="white-space: nowrap;">
  搜索个数：
</div>
        <input
            type="range"
            min="1"
            max="100"
            bind:value={limit}
            step="1"
            id="slider3"
            aria-label="Example slider, range 0 to 100"
        />
        <input
            class="p-slider__input"
            type="text"
            maxlength="3"
            id="slider3-input"
            tabindex="0"
        />
    </div>
    <script>
        var isWebkit =
            /Chrome/i.test(navigator.userAgent) ||
            /Safari/i.test(navigator.userAgent);

        var PROGRESS_COLOUR = "#06c";
        var EMPTY_COLOUR = "#D9D9D9";

        /**
 Renders gradient to fake progress color in webkit browsers.
 @param {HTMLElement} slider Slider element to render background on.
*/
        function renderSlider(slider) {
            if (isWebkit) {
                var value =
                    (slider.value - slider.min) / (slider.max - slider.min);
                slider.style.backgroundImage =
                    "-webkit-gradient(linear, left top, right top, color-stop(" +
                    value +
                    ", " +
                    PROGRESS_COLOUR +
                    "), color-stop(" +
                    value +
                    ", " +
                    EMPTY_COLOUR +
                    "))";
            }
        }

        function equaliseValues(receive, give) {
            receive.value = give.value;
            give.value = receive.value;
        }

        /**
  Attaches change listener to sliders to update their background color.
  @param {HTMLElement} slider Slider element to render background on.
*/
        function initSlider(slider) {
            var input = document.getElementById(slider.id + "-input");
            renderSlider(slider);

            if (input) {
                // Synchronise values of input and slider
                equaliseValues(input, slider);
                input.addEventListener("input", function () {
                    if (!input.value) {
                        input.value = 0;
                    }
                    equaliseValues(slider, input);
                    renderSlider(slider);
                });
            }

            slider.addEventListener("input", function () {
                if (input) {
                    equaliseValues(input, slider);
                }
                renderSlider(slider);
            });
        }

        // Setup all sliders on the page.
        var sliders = document.querySelectorAll("input[type=range]");

        for (var i = 0, l = sliders.length; i < l; i++) {
            initSlider(sliders[i]);
        }
    </script>
    {#if result.length !== 0}
        <table>
            <thead>
                <tr>
                    <th>歌曲名</th>
                    <th>歌手</th>
                    <th
                        ><button
                            class="p-button--positive"
                            on:click={downloadAll}
                        >
                            全部下载</button
                        ></th
                    >
                </tr>
            </thead>
            <tbody>
                {#each result as a}
                    <tr>
                        <th>{a.name}</th>
                        <td>{a.artist}</td>
                        <td>
                            {#if FlagDB2.get(getId(a.url)) === true}
                                <div class="p-status-label--positive">下过了</div>
                            {:else if !a.status}
                                <button
                                    class="p-button"
                                    on:click={() => {
                                        addQueue(a);
                                    }}>下载</button
                                >
                            {:else if a.status == "downloading"}
                                <div class="p-status-label--information">
                                    下载中
                                </div>
                            {:else if a.status == "waiting"}
                                <div class="p-status-label">等待中</div>
                            {:else if a.status == "Done"}
                                <div class="p-status-label--positive">成功</div>
                            {:else}
                                <button
                                    class="p-button--negative"
                                    on:click={() => {
                                        addQueue(a);
                                    }}>重试</button
                                >
                            {/if}
                        </td>
                    </tr>
                {/each}
            </tbody>
        </table>
    {/if}
</div>
