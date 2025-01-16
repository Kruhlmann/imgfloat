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

let last_mouse_move_ms = 0;
let frames = 0;
let fps = 0;
let last_draw_time_ms = window.performance.now();
/** @type {Array.<ImageContext>} */
let image_store = [];
/** @type {CanvasRenderingContext2D} */
let ctx;
/** @type {HTMLCanvasElement} */
let canvas;

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

    for (const image_context of image_store) {
        ctx.drawImage(image_context.image, 0, 0, image_context.width, image_context.height);
    }

    if (ms_now - last_mouse_move_ms < 2000) {
        ctx.font = "48px sans-serif";
        ctx.fillStyle = "lime";
        ctx.fillText(fps.toString(), 10, 48, canvas.width);
    }

    frames++;
}

function update_fps() {
    fps = frames;
    frames = 0;
}

function cleanup_images() {
    const now = new Date().getTime();
    image_store = image_store.filter((i) => {
        if (now > i.expires) {
            console.debug("image expired")
            return false;
        }
        return true
    });
}

function resize_canvas() {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}

function on_websocket_message(event) {
    if (event.data.startsWith("RDY")) {
        document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));
        console.debug("connection is ready");
    } else {
        const response = JSON.parse(event.data)
        if (response.blob) {
            const image = new Image();
            image.onload = () => {
                const width_scale = MAX_IMAGE_WIDTH / image.width;
                const height_scale = MAX_IMAGE_HEIGHT / image.height;
                const scale = Math.min(width_scale, height_scale, 1.0);
                const width = Math.round(scale * image.width);
                const height = Math.round(scale * image.height);
                const expires = new Date().getTime() + 3000;
                image_store.push({ image, width, height, expires });
            };
            image.src = `data:${response.blob.mime_type};base64,${response.blob.bytes_base64}`;
        }
    }
}

document.addEventListener("DOMContentLoaded", async () => {
    const username = window.location.hash.substring(1);
    if (!username) {
        window.location.href = "/";
        return;
    }

    let protocol = window.location.protocol === "https:" ? "wss" : "ws";
    let hostname = window.location.port === "" ? window.location.hostname : `${window.location.hostname}:${window.location.port}`;
    let socket_url = `${protocol}://${hostname}/ws/chat/${username}`;
    console.log(`connecting to ${socket_url}`);
    const socket = new WebSocket(socket_url);
    socket.onmessage = on_websocket_message;

    canvas = document.getElementById("imgfloat");
    ctx = canvas.getContext("2d");
    canvas.onmousemove = () => last_mouse_move_ms = window.performance.now();
    window.onresize = resize_canvas;
    resize_canvas();
    draw();
    setInterval(update_fps, 1000);
    setInterval(cleanup_images, 1000);
});
