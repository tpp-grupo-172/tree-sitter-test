import math
import method_test

def return_pi():
    return math.pi

def return_fake_pi():
    fake_math = method_test.MathClass()
    return fake_math.pi()