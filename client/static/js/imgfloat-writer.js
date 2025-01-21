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
let assets = [];

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

    // for (const image_context of image_store) {
    //     ctx.drawImage(image_context.image, 0, 0, image_context.width, image_context.height);
    // }

    if (ms_now - last_mouse_move_ms < 2000) {
        ctx.font = "48px sans-serif";
        ctx.fillStyle = "lime";
        ctx.fillText(`#${TWITCH_CHANNEL} (${fps} fps)`, 10, 48, canvas.width);
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

    let protocol = window.location.protocol === "https:" ? "wss" : "ws";
    let hostname = window.location.port === "" ? window.location.hostname : `${window.location.hostname}:${window.location.port}`;
    let socket_url = `${protocol}://${hostname}/ws/write/${TWITCH_CHANNEL}`;
    console.log(`connecting to ${socket_url}`);
    const socket = new WebSocket(socket_url);

    canvas = document.getElementById("imgfloat");
    ctx = canvas.getContext("2d");
    canvas.onmousemove = () => last_mouse_move_ms = window.performance.now();
    window.onresize = resize_canvas;
    resize_canvas();
    draw();
    setInterval(update_fps, 1000);

    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));
    assets = await fetch(`/api/assets/${TWITCH_CHANNEL}`).then((r) => r.json());

});
