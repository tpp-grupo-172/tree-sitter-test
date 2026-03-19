import math
import math_utils
from math_utils import add, subtract

def area_of_circle(r = 0):
    return math.pi * r * r

def hypotenuse(a: int, b: int = 1) -> float:
    return math.sqrt(a**2 + b**2)

def volume(a, b: int, c: int = 0, d = 2) -> float:
    return a*b*c*d

def add_values(x: int, y: int) -> int:
    return add(x, y)

def subtract_values(x: int, y: int) -> int:
    return math_utils.subtract(x, y)
class Geometry:
    def __init__(self, shape_name: str):
        self.shape_name = shape_name

    def describe(self) -> None:
        print(f"Shape: {self.shape_name}")

    def area(self, r: float = 0) -> float:
        return area_of_circle(r)

if __name__ == "__main__":
    g = Geometry("circle")
    g.describe()
    print("Area:", area_of_circle(5))
    print("Hypotenuse:", hypotenuse(3, 4))
    print("Sum:", add_values(10, 5))
    print("Difference:", subtract_values(10, 5))
