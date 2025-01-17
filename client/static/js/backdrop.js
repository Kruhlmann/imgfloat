document.addEventListener("DOMContentLoaded", () => {
    const canvas = document.getElementById('demo-backdrop');
    const ctx = canvas.getContext('2d');

    function resizeCanvas() {
        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
    }
    window.addEventListener('resize', resizeCanvas);
    resizeCanvas();

    let width = canvas.width;
    let height = canvas.height;

    const polygonCount = 15;
    const polygonMinSize = 20;
    const polygonMaxSize = 120;
    const polygonMinVertices = 3;
    const polygonMaxVertices = 6;
    const polygonColors = [
        '#e6194B', // Red
        '#3cb44b', // Green
        '#ffe119', // Yellow
        '#4363d8', // Blue
        '#f58231', // Orange
        '#911eb4', // Purple
        '#42d4f4', // Light Blue/Cyan
        '#f032e6', // Magenta
        '#bfef45', // Lime
        '#fabebe', // Pink
        '#469990', // Teal
        '#dcbeff', // Lavender
        '#9A6324', // Brown
        '#fffac8', // Light Yellow
        '#800000', // Maroon
        '#aaffc3', // Mint
        '#808000', // Olive
        '#ffd8b1', // Peach
        '#000075', // Navy
    ];

    class Polygon {
        constructor(spawnInside = false) {
            width = canvas.width;
            height = canvas.height;

            this.size = rand_range(POLYGON_MIN_SIZE_PX, POLYGON_MAX_SIZE_PX);
            this.vertices = this.generate_vertices();
            this.color = POLYGON_COLORS[Math.floor(Math.random() * POLYGON_COLORS.length)];

            if (spawnInside) {
                this.x = Math.random() * width;
                this.y = Math.random() * height;
                const angle = Math.random() * Math.PI * 2;
                const speed = Math.random() * 8 + 1;
                this.vx = Math.cos(angle) * speed;
                this.vy = Math.sin(angle) * speed;
            } else {
                const side = Math.floor(Math.random() * 4);
                if (side === 0) {
                    this.x = Math.random() * width;
                    this.y = -this.size * 2;
                    this.vx = (Math.random() - 0.5) * 2;
                    this.vy = Math.random() * 2 + 1;
                } else if (side === 1) {
                    this.x = width + this.size * 2;
                    this.y = Math.random() * height;
                    this.vx = -(Math.random() * 2 + 1);
                    this.vy = (Math.random() - 0.5) * 2;
                } else if (side === 2) {
                    this.x = Math.random() * width;
                    this.y = height + this.size * 2;
                    this.vx = (Math.random() - 0.5) * 2;
                    this.vy = -(Math.random() * 2 + 1);
                } else {
                    this.x = -this.size * 2;
                    this.y = Math.random() * height;
                    this.vx = Math.random() * 2 + 1;
                    this.vy = (Math.random() - 0.5) * 2;
                }
            }

            this.rotation = Math.random() * Math.PI * 2;
            this.rotationSpeed = (Math.random() - 0.5) * 0.05;
        }

        generate_vertices() {
            const vertex_count = Math.floor(Math.random() * (POLYGON_MAX_VERTICES_COUNT - POLYGON_MIN_VERTICES_COUNT + 1)) + POLYGON_MIN_VERTICES_COUNT;
            const angle_step = (Math.PI * 2) / vertex_count;
            const vertices = [];
            for (let i = 0; i < vertex_count; i++) {
                const angle = i * angle_step;
                const radius = this.size * (0.8 + Math.random() * 0.4);
                const vx = Math.cos(angle) * radius;
                const vy = Math.sin(angle) * radius;
                vertices.push([vx, vy]);
            }
            return vertices;
        }

        update() {
            this.x += this.vx;
            this.y += this.vy;
            this.rotation += this.rotationSpeed;
        }

        is_off_screen() {
            return (this.x < -this.size * 3 ||
                this.x > width + this.size * 3 ||
                this.y < -this.size * 3 ||
                this.y > height + this.size * 3);
        }

        draw(ctx) {
            ctx.save();
            ctx.translate(this.x, this.y);
            ctx.rotate(this.rotation);
            ctx.beginPath();
            ctx.moveTo(this.vertices[0][0], this.vertices[0][1]);
            for (let i = 1; i < this.vertices.length; i++) {
                ctx.lineTo(this.vertices[i][0], this.vertices[i][1]);
            }
            ctx.closePath();
            ctx.fillStyle = this.color;
            ctx.fill();
            ctx.restore();
        }
    }

    const polygons = [];
    for (let i = 0; i < NUMBER_OF_POLYGONS; i++) {
        polygons.push(new Polygon(true));
    }

    let time_of_last_frame = 0;
    const target_fps = 60;
    const fpsInterval = 1000 / target_fps;

    function animate(timestamp) {
        requestAnimationFrame(animate);
        const delta = timestamp - time_of_last_frame;
        if (delta < fpsInterval) {
            return;
        }
        time_of_last_frame = timestamp;

        width = canvas.width;
        height = canvas.height;

        ctx.clearRect(0, 0, width, height);

        for (let i = polygons.length - 1; i >= 0; i--) {
            const poly = polygons[i];
            poly.update();
            poly.draw(ctx);

            if (poly.is_off_screen()) {
                polygons.splice(i, 1);
                polygons.push(new Polygon(false));
            }
        }
    }

    requestAnimationFrame(animate);

    function rand_range(min, max) {
        return Math.random() * (max - min) + min;
    }
});
