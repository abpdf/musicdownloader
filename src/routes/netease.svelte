<script>
    import { invoke } from "@tauri-apps/api/core";
    let result = $state([]);
    let name = $state();
    async function search() {
        result =[];
        const url = `https://api.qijieya.cn/meting/?type=search&id=${encodeURIComponent(name)}&limit=30&server=netease`;
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
    if (typeof name !== 'string') return '';

    // 第一步：将 / 和 \ 全部替换为 &（and 符号）
    let result = name.replace(/[\\\/]/g, '&');

    // 第二步：移除其他非法字符（如 : * ? " < > |）
    // 注意：& 已被保留，不会在这里被移除
    result = result.replace(/[:*?"<>|]/g, '');

    // 第三步：去除首尾空格（Windows 不允许文件名以空格结尾）
    return result.trim();
}
    async function getFinalMp3Url(apiUrl) {

        const response = await fetch(apiUrl, { method: "HEAD" });
        // response.url 此时就是最后的 MP3 直链，没有下载任何文件内容
        return response.url;
    }
    async function download(u) {
        u.status="downloading";
        const finalurl = await getFinalMp3Url(u.url);
        const status = await invoke("download_file_async", { url: finalurl, name: sanitizeFileName(u.name+"-"+u.artist)});
        u.status=status;
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
    {#if result.length !==0 }
    <table>
        <thead>
            <tr>
                <th>歌曲名</th>
                <th>歌手</th>
                <th><button class="p-button--positive">全部下载</button></th>
            </tr>
        </thead>
        <tbody>
            {#each result as a}
                <tr>
                    <th>{a.name}</th>
                    <td>{a.artist}</td>
                    <td>
                    {#if !a.status}
                        <button class="p-button" on:click={() =>{download(a);}}>下载</button>
                    {:else if a.status=="downloading"}
                        <div class="p-status-label--information">下载中</div>
                    {:else if a.status=="wating"}
                        <div class="p-status-label">等待中</div>
                    {:else if a.status=="Done"}
                        <div class="p-status-label--positive">成功</div>
                    {:else}
                        <button class="p-button--negative" on:click={() =>{download(a);}}>重试</button>
                    {/if}
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
    {/if}
</div>
