<script>
    // ============ 方案二：独立键名（每个ID单独存） ============
    const FlagDB2 = {
        // 生成唯一键名（统一加前缀，方便管理）
        _getKey(id) {
            return "flag_" + id; // 例如：flag_1001
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
                if (key && key.startsWith("flag_")) {
                    keysToRemove.push(key);
                }
            }
            keysToRemove.forEach((key) => localStorage.removeItem(key));
        },
    };

    import { page } from "./page.svelte.js";
    import { fade } from "svelte/transition";
    import { invoke } from "@tauri-apps/api/core";
    let result = $state([]);
    let name = $state();
    let limit = $state(30);
    let showInofOfStatus = $state(false);
    let isBusy = $state(false);
    let reset = $state(false);
    let queue = [];
    let searchInfo = $state("");
    let br = $state(+localStorage.getItem("br") || 320);
    let availableBr = [2000, 320, 192, 128];
    let searchType = $state("单曲");
    let availablesSearchType = ["单曲", "歌单"];

    async function search() {
        result = [];
        searchInfo = "";
        const url = `https://api.qijieya.cn/meting/?type=search&id=${encodeURIComponent(name)}&limit=${limit}&server=netease`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`HTTP 错误! 状态码: ${response.status}`);
            }
            result = await response.json();
        } catch (error) {
            searchInfo = "" + error;
        }
    }
    async function searchPlaylist() {
        result = [];
        searchInfo = "";
        const url = `https://163api.qijieya.cn/cloudsearch?keywords=${encodeURIComponent(name)}&limit=${limit}&type=1000`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`HTTP 错误! 状态码: ${response.status}`);
            }
            result = await response.json();
        } catch (error) {
            searchInfo = "" + error;
        }
    }
    async function getDetail(a) {
        const url = `https://api.qijieya.cn/meting/?type=playlist&id=${a.id}`;
        try {
            const response = await fetch(url);
            if (!response.ok) {
                throw new Error(`HTTP 错误! 状态码: ${response.status}`);
            }
            a["详情"] = await response.json();
        } catch (error) {
            a["详情"] = "" + error;
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
    function getId(url) {
        const parsed = new URL(url);
        return parsed.searchParams.get("id");
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

    function downloadAll(result) {
        for (let i = 0; i < result.length; i++) {
            if (
                result[i].status != "Done" &&
                result[i].status != "waiting" &&
                result[i].status != "downloading" &&
                FlagDB2.get(getId(result[i].url)) !== true
            ) {
                addQueue(result[i]);
            }
        }
    }

    async function doWork() {
        isBusy = true;
        while (queue.length > 0) {
            try {
                await download(queue[0]);
            } catch (error) {
                queue[0].status = "" + error;
            }
            queue.shift();
        }
        isBusy = false;
    }

    async function download(u) {
        u.status = "downloading";
        console.log(u.url + `&br=${br}`);
        const status = await invoke("download_file_async_without_redirect", {
            url: u.url + `&br=${br}`,
            name: sanitizeFileName(u.name + "-" + u.artist),
        });
        if (status === "Done") {
            FlagDB2.set(getId(u.url), true);
        }
        u.status = status;
    }
</script>

<div style="margin-bottom: 1rem;">
    <button
        class="p-button"
        on:click={() => {
            page.num = 0;
        }}
        disabled={isBusy}>← 返回</button
    >
</div>
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
        {#if searchType === "单曲"}
            <button
                type="submit"
                class="p-search-box__button"
                on:click={search}
                disabled={isBusy}
            >
                <i class="p-icon--search">Search</i>
            </button>
        {:else if searchType === "歌单"}
            <button
                type="submit"
                class="p-search-box__button"
                on:click={searchPlaylist}
                disabled={isBusy}
            >
                <i class="p-icon--search">Search</i>
            </button>
        {/if}
    </form>
    <div>
        <p style="margin-bottom:0px;">
            搜索种类：
            {#each availablesSearchType as current}
                {#if searchType === current}
                    <button
                        class="p-button--positive"
                        on:click={() => {
                            searchType = current;
                        }}
                        disabled={isBusy}>{current}</button
                    >
                {:else}
                    <button
                        class="p-button"
                        on:click={() => {
                            searchType = current;
                            result = [];
                        }}
                        disabled={isBusy}>{current}</button
                    >
                {/if}
            {/each}
        </p>
    </div>
    <div class="p-slider__wrapper" style="margin-bottom: 0.5rem;">
        <div style="white-space: nowrap;">搜索个数：</div>
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
    <div>
        <p style="margin-bottom:0px;">
            下载码率：
            {#each availableBr as current}
                {#if br === current}
                    <button
                        class="p-button--positive"
                        on:click={() => {
                            br = current;
                            localStorage.setItem("br", current);
                        }}>{current}</button
                    >
                {:else}
                    <button
                        class="p-button"
                        on:click={() => {
                            br = current;
                            localStorage.setItem("br", current);
                        }}>{current}</button
                    >
                {/if}
            {/each}
        </p>
    </div>
    <div>
        <div
            style="cursor: pointer; user-select: none; display: flex; justify-content: space-between; align-items: center; padding: 0.5rem 0;"
            on:click={() => {
                showInofOfStatus = !showInofOfStatus;
            }}
        >
            <strong>关于下载记录</strong>
            <span>{showInofOfStatus ? "▾" : "▸"}</span>
        </div>
        {#if showInofOfStatus}
            <div
                in:fade={{ duration: 400 }}
                style="padding-left: 1rem; color: #666;"
            >
                <p>
                    仅记录哪个下载了，以标记为“下过了”，避免重复下载。不同板块记录不互通。重置后立即生效，此操作不可逆！
                </p>
                <p>
                    <button
                        class="p-button--negative"
                        on:click={() => {
                            FlagDB2.clear();
                            reset = !reset;
                        }}>重置全部</button
                    >
                    <button
                        class="p-button--negative"
                        on:click={() => {
                            for (let i = 0; i < result.length; i++) {
                                FlagDB2.delete(getId(result[i].url));
                            }
                            reset = !reset;
                        }}>重置本页</button
                    >
                </p>
            </div>
        {/if}
    </div>
    <script>
        var isWebkit =
            /Chrome/i.test(navigator.userAgent) ||
            /Safari/i.test(navigator.userAgent);

        var PROGRESS_COLOUR = "#06c";
        var EMPTY_COLOUR = "#D9D9D9";

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
    {#if searchInfo}
        <div class="p-notification--negative">
            <div class="p-notification__content">
                <h5 class="p-notification__title">搜索失败</h5>
                <p class="p-notification__message">{searchInfo}</p>
            </div>
        </div>
    {/if}
    {#key reset}
        {#if result.length !== 0}
            <hr
                class="p-divider"
                style="margin-top: 0.5rem;margin-bottom: 0.5rem;"
            />

            {#if searchType === "单曲"}
                <table>
                    <thead>
                        <tr>
                            <th class="p-heading--5">歌曲名</th>
                            <th class="p-heading--5">歌手</th>
                            <th
                                ><button
                                    style="margin-bottom:0px;"
                                    class="p-button--positive"
                                    on:click={()=>{downloadAll(result)}}
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
                                        <div class="p-status-label--positive">
                                            下过了
                                        </div>
                                    {:else if !a.status}
                                        <button
                                            style="margin-bottom:0px;"
                                            class="p-button"
                                            on:click={() => {
                                                addQueue(a);
                                            }}>下载</button
                                        >
                                    {:else if a.status == "downloading"}
                                        <div
                                            class="p-status-label--information"
                                        >
                                            下载中
                                        </div>
                                    {:else if a.status == "waiting"}
                                        <div class="p-status-label">等待中</div>
                                    {:else if a.status == "Done"}
                                        <div class="p-status-label--positive">
                                            成功
                                        </div>
                                    {:else}
                                        <button
                                            style="margin-bottom:0px;"
                                            class="p-button--negative"
                                            on:click={() => {
                                                addQueue(a);
                                            }}>重试</button
                                        >
                                    {/if}
                                </td>
                            </tr>
                            {#if FlagDB2.get(getId(a.url)) !== true && !!a.status && a.status !== "downloading" && a.status !== "waiting" && a.status !== "Done"}
                                <tr style="border-top:0"
                                    ><td colspan="3">
                                        <div class="p-notification--negative">
                                            <div
                                                class="p-notification__content"
                                            >
                                                <h5
                                                    class="p-notification__title"
                                                >
                                                    出错了
                                                </h5>
                                                <p
                                                    class="p-notification__message"
                                                >
                                                    {a.status}
                                                </p>
                                            </div>
                                        </div></td
                                    ></tr
                                >
                            {/if}
                        {/each}
                    </tbody>
                </table>
            {/if}

            {#if searchType === "歌单"}
                <table>
                    <thead>
                        <tr>
                            <th class="p-heading--5">歌单名</th>
                            <th class="p-heading--5">创建者</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each result.result.playlists as current}
                            <tr>
                                <th>{current.name}</th>
                                <td>{current.creator.nickname}</td>
                                <td>
                                    {#if current["详情"]===undefined}
                                        <button
                                            on:click={() => {getDetail(current);}}
                                            class="p-button--base">展开</button
                                        >
                                    {:else if !Array.isArray(current["详情"])}
                                        <button
                                            on:click={() => {getDetail(current);}}
                                            class="p-button--nagative">重试</button
                                        >
                                    {/if}
                                </td>
                            </tr>
                            {#if !Array.isArray(current["详情"])&&current["详情"]!==undefined}
                                <tr style="border-top:0">
                                    <td colspan="3">
                                        <div class="p-notification--negative">
                                            <div
                                                class="p-notification__content"
                                            >
                                                <h5
                                                    class="p-notification__title"
                                                >
                                                    展开出错
                                                </h5>
                                                <p
                                                    class="p-notification__message"
                                                >
                                                    {current["详情"]}
                                                </p>
                                            </div>
                                        </div>
                                    </td>
                                </tr>
                            {:else if Array.isArray(current["详情"])}
                                <tr>
                                    <td colspan="3">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th class="p-heading--5"
                                                        >歌曲名</th
                                                    >
                                                    <th class="p-heading--5"
                                                        >歌手</th
                                                    >
                                                    <th
                                                        ><button
                                                            style="margin-bottom:0px;"
                                                            class="p-button--positive"
                                                            on:click={()=>{downloadAll(current["详情"])}}
                                                        >
                                                            全部下载</button
                                                        ></th
                                                    >
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {#each current["详情"] as a}
                                                    <tr>
                                                        <th>{a.name}</th>
                                                        <td>{a.artist}</td>
                                                        <td>
                                                            {#if FlagDB2.get(getId(a.url)) === true}
                                                                <div
                                                                    class="p-status-label--positive"
                                                                >
                                                                    下过了
                                                                </div>
                                                            {:else if !a.status}
                                                                <button
                                                                    style="margin-bottom:0px;"
                                                                    class="p-button"
                                                                    on:click={() => {
                                                                        addQueue(
                                                                            a,
                                                                        );
                                                                    }}
                                                                    >下载</button
                                                                >
                                                            {:else if a.status == "downloading"}
                                                                <div
                                                                    class="p-status-label--information"
                                                                >
                                                                    下载中
                                                                </div>
                                                            {:else if a.status == "waiting"}
                                                                <div
                                                                    class="p-status-label"
                                                                >
                                                                    等待中
                                                                </div>
                                                            {:else if a.status == "Done"}
                                                                <div
                                                                    class="p-status-label--positive"
                                                                >
                                                                    成功
                                                                </div>
                                                            {:else}
                                                                <button
                                                                    style="margin-bottom:0px;"
                                                                    class="p-button--negative"
                                                                    on:click={() => {
                                                                        addQueue(
                                                                            a,
                                                                        );
                                                                    }}
                                                                    >重试</button
                                                                >
                                                            {/if}
                                                        </td>
                                                    </tr>
                                                    {#if FlagDB2.get(getId(a.url)) !== true && !!a.status && a.status !== "downloading" && a.status !== "waiting" && a.status !== "Done"}
                                                        <tr style="border-top:0"
                                                            ><td colspan="3">
                                                                <div
                                                                    class="p-notification--negative"
                                                                >
                                                                    <div
                                                                        class="p-notification__content"
                                                                    >
                                                                        <h5
                                                                            class="p-notification__title"
                                                                        >
                                                                            出错了
                                                                        </h5>
                                                                        <p
                                                                            class="p-notification__message"
                                                                        >
                                                                            {a.status}
                                                                        </p>
                                                                    </div>
                                                                </div></td
                                                            ></tr
                                                        >
                                                    {/if}
                                                {/each}
                                            </tbody>
                                        </table>
                                    </td>
                                </tr>
                            {/if}
                        {/each}
                    </tbody>
                </table>
            {/if}
        {/if}
    {/key}
</div>
