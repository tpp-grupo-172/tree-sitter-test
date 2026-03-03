export function add(a: number, b: number): number {
    return a + b;
}

export function subtract(a: number, b: number): number {
    return a - b;
}

export function multiply(a: number, b: number): number {
    return a * b;
}

export class Calculator {
    add(a: number, b: number): number {
        return add(a, b);
    }

    subtract(a: number, b: number): number {
        return subtract(a, b);
    }

    multiply(a: number, b: number): number {
        return multiply(a, b);
    }
}