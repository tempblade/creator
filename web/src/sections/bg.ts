interface Point {
    x: number;
    y: number;
    color: string;
    radius: number;
}

const canvas = document.getElementById("landing-bg") as HTMLCanvasElement;

const initialPoints: Array<Point> = [{
    x: 500,
    y: 400,
    color: "purple",
    radius: Math.random() * 200
},
{
    x: 900,
    y: 500,
    color: "#326CCC",
    radius: Math.random() * 200
},
{
    x: 1600,
    y: 400,
    color: "#95B2F5",
    radius: Math.random() * 200
},
{
    x: 1200,
    y: 600,
    color: "#32C3E3",
    radius: Math.random() * 200

},
{
    x: 600,
    y: 500,
    color: "#8AFFAD",
    radius: Math.random() * 200

},
{
    x: 900,
    y: 300,
    color: "purple",
    radius: Math.random() * 200

},
]

export class GradientBackground {
    context: CanvasRenderingContext2D;

    constructor(context: CanvasRenderingContext2D) {
        this.context = context;
        this.draw = this.draw.bind(this);

    }

    draw(time: number) {
        const { width, height } = this.context.canvas;

        this.context.clearRect(0, 0, width, height * 2);


        this.context.save();

        this.context.translate(width * 0.5, height * 0.5);


        const angle = (time * 0.01) % 360;
        const radian = angle * (Math.PI / 180);

        this.context.rotate(radian);

        this.context.translate(width * -0.5, height * -0.5);


        initialPoints.forEach((point, index) => {
            const { color, radius } = point;


            // Calculate the position/angle on a circle by the time
            const angle = time * 0.05 + index * 10 % 360;

            // Convert the angle to radian
            const radian = angle * (Math.PI / 180);

            // Calculate the offset based on the radius
            const offsetX = Math.cos(radian) * radius;
            const offsetY = Math.sin(radian) * radius;

            // Calculate the position based on offset and initial position
            const x = offsetX + point.x;
            const y = offsetY + point.y;

            // Create the gradient
            const gradient = this.context.createRadialGradient(x, y, 0, x, y, 500);

            gradient.addColorStop(0, color);
            gradient.addColorStop(1, "rgba(0,0,0,0)");

            this.context.fillStyle = gradient;
            this.context.globalCompositeOperation = "lighten";


            // Draw a rect with the gradient
            this.context.fillRect(0, 0, width, height);


        })

        this.context.restore();

        return requestAnimationFrame(this.draw);
    }
}

const ctx = canvas.getContext("2d");

if (ctx) {
    const bg = new GradientBackground(ctx);

    requestAnimationFrame(bg.draw);
}

