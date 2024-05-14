class Circle:
    def __init__(self, radius):
        self.radius = radius

    def area(self):
        return 3.141592653 * self.radius * self.radius

circle = Circle(4)
i = 0 
while i <100000:
    print(circle.area())
    i+=1
