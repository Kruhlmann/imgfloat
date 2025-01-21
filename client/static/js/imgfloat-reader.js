/**
 * @typedef ImageContext
 * @type {object}
 * @property {Image} image
 * @property {number} width
 * @property {number} height
 * @property {number} expires
 */

const MAX_IMAGE_WIDTH = 200;
const MAX_IMAGE_HEIGHT = 150;
const TARGET_FPS = 60;
const MS_PER_FRAME = 1000 / TARGET_FPS;
const TWITCH_CHANNEL = window.location.hash.substring(1);

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

function draw() {
    window.requestAnimationFrame(draw);
    const ms_now = window.performance.now();
    const delta_ms = ms_now - last_draw_time_ms
    if (delta_ms < MS_PER_FRAME) {
        return
    }
    const excess_time = delta_ms % MS_PER_FRAME;
    last_draw_time_ms = ms_now - excess_time;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    for (const asset of live_assets) {
        ctx.drawImage(asset.image, asset.x / 100 * canvas.width, asset.y / 100 * canvas.height, asset.w / 100 * canvas.width, asset.h / 100 * canvas.height);
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

document.addEventListener("DOMContentLoaded", async () => {
    if (!TWITCH_CHANNEL) {
        window.location.href = "/";
        return;
    }

    const protocol = window.location.protocol === "https:" ? "wss" : "ws";
    const hostname = window.location.port === "" ? window.location.hostname : `${window.location.hostname}:${window.location.port}`;
    const socket_url = `${protocol}://${hostname}/ws/read/${TWITCH_CHANNEL}`;
    console.log(`connecting to ${socket_url}`);
    socket = new WebSocket(socket_url);
    socket.onmessage = (event) => {
        const state = JSON.parse(event.data);
        live_assets = state.assets.map((a) => {
            const image = new Image();
            image.src = a.url;
            return { x: a.x, y: a.y, w: a.w, h: a.h, image }
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
    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));
});
