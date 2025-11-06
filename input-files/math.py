import math

def area_of_circle(r = 0):
    return math.pi * r * r

def hypotenuse(a: int, b: str = "hola") -> float:
    return math.sqrt(a**2 + b**2)

def volume(a, b: int, c: int = 0, d = 2) -> float:
    return a*b*c*d
class Geometry:
    def __init__(self, shape_name: str):
        self.shape_name = shape_name

    def describe(self):
        print(f"Shape: {self.shape_name}")

if __name__ == "__main__":
    g = Geometry("circle")
    g.describe()
    print("Area:", area_of_circle(5))
