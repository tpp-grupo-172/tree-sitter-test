def add(a: int, b: int) -> int:
    return a + b

def subtract(a: int, b: int) -> int:
    return a - b

def multiply(a: int, b: int) -> int:
    return a * b

class Calculator:
    def add(self, a: int, b: int) -> int:
        return add(a, b)

    def subtract(self, a: int, b: int) -> int:
        return subtract(a, b)

    def multiply(self, a: int, b: int) -> int:
        return multiply(a, b)