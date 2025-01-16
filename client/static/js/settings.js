document.addEventListener("DOMContentLoaded", async () => {
    const session = await fetch("/api/whoami").then((r) => r.json()).catch(() => undefined);
    if (!session) {
        window.location.href = "/";
        return;
    }
    document.querySelectorAll("name").forEach((n) => n.innerHTML = session.display_name);
    document.querySelectorAll(".is-loading").forEach((n) => n.classList.remove("is-loading"));
});
