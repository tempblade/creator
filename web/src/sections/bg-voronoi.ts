interface Point {
  x: number;
  y: number;
  color: string;
}

interface VoronoiCell {
  site: Point;
  vertices: Point[];
}

function generateVoronoiPattern(
  canvas: HTMLCanvasElement,
  points: Array<Point>
) {
  const ctx = canvas.getContext("2d");

  if (ctx) {
    // Draw Voronoi regions
    for (let x = 0; x < canvas.width; x++) {
      for (let y = 0; y < canvas.height; y++) {
        let closestPointIndex = 0;
        let closestDistance = distance(x, y, points[0].x, points[0].y);

        for (let i = 1; i < points.length; i++) {
          const dist = distance(x, y, points[i].x, points[i].y);
          if (dist < closestDistance) {
            closestDistance = dist;
            closestPointIndex = i;
          }
        }

        const { x: px, y: py, color } = points[closestPointIndex];
        ctx.fillStyle = color;
        ctx.fillRect(x, y, 1, 1);
      }
    }
  }
}

function distance(x1: number, y1: number, x2: number, y2: number) {
  const dx = x2 - x1;
  const dy = y2 - y1;
  return Math.sqrt(dx * dx + dy * dy);
}

// Get canvas element and generate Voronoi pattern
const canvas = document.getElementById("landing-bg") as HTMLCanvasElement;


const initialPoints: Array<Point> = [{
  x: 200,
  y: 200,
  color: "#8AFFAD",
},
{
  x: 800,
  y: 500,
  color: "#326CCC",
},
{
  x: 1100,
  y: 300,
  color: "#95B2F5",
},
{
  x: 1200,
  y: 600,
  color: "#32C3E3",
},
{
  x: 300,
  y: 900,
  color: "purple",
},]


const draw = (time: number) => {
  const radius = 200;

  const angle = time % 360;

  const radian = angle * (Math.PI / 180);

  const x = Math.cos(radian) * radius;
  const y = Math.sin(radian) * radius;

  const points = initialPoints.map((p) => ({ ...p, x: p.x + x, y: p.y + y }));

  generateVoronoiPattern(canvas, points);

  //requestAnimationFrame(draw);
}

requestAnimationFrame(draw);