const TWITCH_CHANNEL = window.location.hash.substring(1);
const HOSTNAME = window.location.hostname;

let loaded = false;
let last_mouse_move_ms = 0;
let frames = 0;
let fps = 0;
let last_draw_time_ms = window.performance.now();
/** @type {CanvasRenderingContext2D} */
let ctx;
/** @type {HTMLCanvasElement} */
let canvas;
/** @type {WebSocket} */
let socket;
let live_assets = [];
let selected_asset_id;
let is_dragging = false;
let ms_per_frame;


function draw() {
    window.requestAnimationFrame(draw);
    const ms_now = window.performance.now();
    const delta_ms = ms_now - last_draw_time_ms
    if (delta_ms < ms_per_frame) {
        return
    }
    const excess_time = delta_ms % ms_per_frame;
    last_draw_time_ms = ms_now - excess_time;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (const asset of live_assets) {
        const adjust_x = (asset.x / 100) * canvas.width;
        const adjust_y = (asset.y / 100) * canvas.height;
        const adjust_w = (asset.w / 100) * canvas.width;
        const adjust_h = (asset.h / 100) * canvas.height;

        ctx.drawImage(
            asset.image,
            adjust_x,
            adjust_y,
            adjust_w,
            adjust_h
        );

        if (asset.id === selected_asset_id) {
            ctx.strokeStyle = "orange";
            ctx.lineWidth = 4;
            ctx.strokeRect(adjust_x, adjust_y, adjust_w, adjust_h);
        }
    }

    if (ms_now - last_mouse_move_ms < 2000) {
        ctx.font = "24px sans-serif";
        ctx.fillStyle = "#ebdbb2";
        ctx.fillText(`#${TWITCH_CHANNEL} (${fps} fps)`, 10, 24, canvas.width);
        ctx.fillText("Press 'q' to open settings", 10, 48, canvas.width);
        ctx.fillText("Press 'a' to open asset library", 10, 72, canvas.width);
    }

    frames++;
}

function update_fps() {
    fps = frames;
    frames = 0;
}

function resize_canvas() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

function close_assets_window() {
    document.getElementById("assets").classList.remove("show");
}

function update_dom_with_new_settings(settings) {
    document.getElementById("settings.fps-target").value = settings.fps_target;
    document.getElementById("settings.background-opacity").value = settings.background_opacity;
    document.getElementById("dim").style.backgroundColor = `rgba(0, 0, 0, ${(100 - settings.background_opacity) / 100})`;
    ms_per_frame = 1000 / settings.fps_target;
    document.getElementById("settings").classList.remove("show");
}

async function save_settings() {
    await fetch("/api/settings", {
        headers: { "Content-Type": "application/json" },
        method: "PUT",
        body: JSON.stringify({
            fps_target: Number.parseInt(document.getElementById("settings.fps-target").value, 10),
            background_opacity: Number.parseInt(document.getElementById("settings.background-opacity").value, 10),
        }),
    })
        .then((r) => {
            if (r.status !== 201 && r.status !== 200) {
                throw r.status;
            }
            return r;
        })
        .then((r) => r.json())
        .catch(alert)
        .then(update_dom_with_new_settings);
}

function cancel_settings() {
    document.getElementById("settings").classList.remove("show");
}

function delete_selected_asset() {
    socket.send(JSON.stringify({ Delete: selected_asset_id }))
    live_assets = live_assets.filter((a) => a.id !== selected_asset_id);
    selected_asset_id = undefined;
}

function remote_state_full() {
    const state = {
        assets: live_assets.map((a) => {
            return { id: a.id, x: a.x, y: a.y, w: a.w, h: a.h, url: a.image.src }
        })
    }
    console.log("Sending new state");
    console.log(state);
    socket.send(JSON.stringify({ New: state }))
}

function remote_state_update() {
    const selected_asset = live_assets.find((a) => a.id === selected_asset_id);
    const payload = {
        id: selected_asset.id,
        x: selected_asset.x,
        y: selected_asset.y,
        w: selected_asset.w,
        h: selected_asset.h,
        url: selected_asset.image.src,
    };
    socket.send(JSON.stringify({ Update: payload, }))
}

async function add_asset(filename) {
    const x = Math.random() * 100;
    const y = Math.random() * 100;
    const w = Math.random() * (20 - 10) + 10;
    const h = Math.random() * (20 - 10) + 10;;
    const id = window.crypto.randomUUID();
    const image = new Image();
    image.src = `/api/assets/${TWITCH_CHANNEL}/${filename}`;
    console.log(`Adding image ${image.src}`)
    live_assets.push({ id, image, x, y, w, h })
    selected_asset_id = id;
    remote_state_full()
}

async function refresh_file_list() {
    let image_list = await fetch(`/api/assets/${TWITCH_CHANNEL}`).then((r) => r.json());
    image_list.sort();
    let image_list_html = image_list
        .map((asset) => `<div class="asset"><i class="bi bi-file-earmark-image"></i><span onclick="add_asset('${asset.filename}')">${asset.filename}</span></div>`)
        .join("\n");
    let audio_list_html = "";
    let video_list_html = "";
    document.getElementById("asset-list").innerHTML = `
        <section class="images">
            <h2>Images</h2>
            ${image_list_html}
        </section>
        <section class="audio">
            <h2>Audio</h2>
            ${audio_list_html}
        </section>
        <section class="video">
            <h2>Video</h2>
            ${video_list_html}
        </section>
    `;
}

function open_file_dialog() {
    document.getElementById("asset-upload-file").click();
}

async function upload_asset() {
    const file = document.getElementById("asset-upload-file").files[0];
    if (!file) {
        return;
    }
    const method = "POST";
    const body = new FormData();
    body.append('file', file);
    await fetch(`/api/assets/${TWITCH_CHANNEL}`, { method, body })
        .then(response => {
            if (!response.ok) {
                throw new Error(`Server error: ${response.status}`);
            }
            return response.text();
        })
        .catch(error => {
            console.error('Error uploading file:', error);
        });
    await refresh_file_list();
}

document.addEventListener("keyup", (event) => {
    if (event.key === "q") {
        document.getElementById("settings").classList.add("show");
    } else if (event.key === "a") {
        document.getElementById("assets").classList.add("show");
    } else if (event.key === "Delete") {
        delete_selected_asset();
    }
});

document.addEventListener("DOMContentLoaded", async () => {
    if (!TWITCH_CHANNEL) {
        window.location.href = "/";
        throw new Error();
    }
    await fetch("/api/settings")
        .then((r) => r.json())
        .catch(() => {
            window.location.href = "/"
            throw new Error();
        })
        .then(update_dom_with_new_settings);

    const protocol = window.location.protocol === "https:" ? "wss" : "ws";
    const hostname = window.location.port === "" ? window.location.hostname : `${window.location.hostname}:${window.location.port}`;
    const socket_url = `${protocol}://${hostname}/ws/write/${TWITCH_CHANNEL}`;
    console.log(`connecting to ${socket_url}`);
    socket = new WebSocket(socket_url);
    socket.onmessage = (event) => {
        selected_asset_id = undefined;
        const state = JSON.parse(event.data);
        live_assets = state.assets.map((a) => {
            const image = new Image();
            image.src = a.url;
            return { id: a.id, x: a.x, y: a.y, w: a.w, h: a.h, image }
        })
    }
    socket.onclose = () => socket = new WebSocket(socket_url);
    socket.onerror = () => socket = new WebSocket(socket_url);

    canvas = document.getElementById("imgfloat");
    ctx = canvas.getContext("2d");
    canvas.onmousemove = () => last_mouse_move_ms = window.performance.now();
    window.onresize = resize_canvas;
    resize_canvas();
    draw();
    setInterval(update_fps, 1000);

    refresh_file_list();
    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));
    document.getElementById("twitch-iframe").setAttribute("src", `https://player.twitch.tv/?channel=${TWITCH_CHANNEL}&autoplay=true&muted=true&parent=${HOSTNAME}`);

    canvas.addEventListener("click", (event) => {
        const rect = canvas.getBoundingClientRect();
        const click_x = event.clientX - rect.left;
        const click_y = event.clientY - rect.top;
        for (let i = live_assets.length - 1; i >= 0; i--) {
            const asset = live_assets[i];
            const adjust_x = (asset.x / 100) * canvas.width;
            const adjust_y = (asset.y / 100) * canvas.height;
            const adjust_w = (asset.w / 100) * canvas.width;
            const adjust_h = (asset.h / 100) * canvas.height;
            if (
                click_x >= adjust_x &&
                click_x <= adjust_x + adjust_w &&
                click_y >= adjust_y &&
                click_y <= adjust_y + adjust_h
            ) {
                selected_asset_id = asset.id;
                return;
            }
        }
        selected_asset_id = undefined;
    })
    canvas.addEventListener("mousedown", (event) => {
        const rect = canvas.getBoundingClientRect();
        const click_x = event.clientX - rect.left;
        const click_y = event.clientY - rect.top;
        if (selected_asset_id) {
            const selected_asset = live_assets.find((a) => a.id === selected_asset_id);
            const adjust_x = (selected_asset.x / 100) * canvas.width;
            const adjust_y = (selected_asset.y / 100) * canvas.height;
            const adjust_w = (selected_asset.w / 100) * canvas.width;
            const adjust_h = (selected_asset.h / 100) * canvas.height;

            if (
                click_x >= adjust_x &&
                click_x <= adjust_x + adjust_w &&
                click_y >= adjust_y &&
                click_y <= adjust_y + adjust_h
            ) {
                is_dragging = true;
                drag_x_off = click_x - adjust_x;
                drag_y_off = click_y - adjust_y;
            }
        }
    });

    canvas.addEventListener("mousemove", (event) => {
        if (is_dragging && selected_asset_id) {
            const rect = canvas.getBoundingClientRect();
            const mouseX = event.clientX - rect.left;
            const mouseY = event.clientY - rect.top;
            const new_x = mouseX - drag_x_off;
            const new_y = mouseY - drag_y_off;

            const selected_asset = live_assets.find((a) => a.id === selected_asset_id);
            selected_asset.x = (new_x / canvas.width) * 100;
            selected_asset.y = (new_y / canvas.height) * 100;

            remote_state_update();
        }
    });

    canvas.addEventListener("mouseup", () => {
        is_dragging = false;
    });

});
