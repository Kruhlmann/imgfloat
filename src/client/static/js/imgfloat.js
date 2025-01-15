const MAX_IMAGE_WIDTH = 200;
const MAX_IMAGE_HEIGHT = 150;

document.addEventListener("DOMContentLoaded", async () => {
    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));

    const canvas = document.getElementById('imgfloat');
    const ctx = canvas.getContext('2d');

    const socket = new WebSocket("ws://localhost:3000/ws/chat/gasolinebased");
    socket.onmessage = (event) => {
        if (event.data.startsWith("RDY")) {
            console.debug("Connection is ready");
        } else {
            const response = JSON.parse(event.data)
            if (response.blob) {
                const image = new Image();
                image.onload = () => {
                    const origWidth = image.width;
                    const origHeight = image.height;
                    const widthScale = MAX_IMAGE_WIDTH / origWidth;
                    const heightScale = MAX_IMAGE_HEIGHT / origHeight;
                    const scale = Math.min(widthScale, heightScale, 1.0);
                    const scaledWidth = Math.round(origWidth * scale);
                    const scaledHeight = Math.round(origHeight * scale);
                    ctx.drawImage(image, 0, 0, scaledWidth, scaledHeight);
                };
                image.src = `data:${response.blob.mime_type};base64,${response.blob.bytes_base64}`;
            }
        }
    }
});
