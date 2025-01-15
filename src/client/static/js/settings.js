document.addEventListener("DOMContentLoaded", async () => {
    const session = await fetch("/whoami").then((r) => r.json()).catch(() => undefined);
    if (!session) {
        window.location.href = "/";
        return;
    }
    document.querySelectorAll("name").forEach((n) => n.innerHTML = session.display_name);
    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));

    const socket = new WebSocket("/ws/chat");
    socket.onmessage = (event) => {
        if (event.data.startsWith("RDY")) {
            console.debug("Connection is ready");
        } else {
            console.log(`message received`, event.data)
        }
    }
});
