window.begin_render = () => {
    console.log(1)
    requestAnimationFrame(draw)
}

function draw() {
    console.log(2)
}
