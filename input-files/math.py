import math

def area_of_circle(r):
    return math.pi * r * r

def hypotenuse(a, b):
    return math.sqrt(a**2 + b**2)

class Geometry:
    def __init__(self, shape_name):
        self.shape_name = shape_name

    def describe(self):
        print(f"Shape: {self.shape_name}")

if __name__ == "__main__":
    g = Geometry("circle")
    g.describe()
    print("Area:", area_of_circle(5))
