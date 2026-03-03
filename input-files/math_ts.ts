import * as math_utils from './math_utils';
import { add, subtract } from './math_utils';

export function areaOfCircle(r: number = 0): number {
    return Math.PI * r * r;
}

export function hypotenuse(a: number, b: number = 1): number {
    return Math.sqrt(a ** 2 + b ** 2);
}

export function volume(a: number, b: number, c: number = 1, d: number = 2): number {
    return a * b * c * d;
}

export function addValues(x: number, y: number): number {
    return add(x, y);
}

export function subtractValues(x: number, y: number): number {
    return math_utils.subtract(x, y);
}

export class Geometry {
    shapeName: string;

    constructor(shapeName: string) {
        this.shapeName = shapeName;
    }

    describe(): void {
        console.log(`Shape: ${this.shapeName}`);
    }

    area(r: number = 0): number {
        return areaOfCircle(r);
    }
}

const g = new Geometry("circle");
g.describe();
console.log("Area:", areaOfCircle(5));
console.log("Hypotenuse:", hypotenuse(3, 4));
console.log("Sum:", addValues(10, 5));
console.log("Difference:", subtractValues(10, 5));