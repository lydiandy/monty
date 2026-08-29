# Nested `def` inside a method capturing `self` (the Counter `inc` combo).
# Upstream tests cover class-in-function and methods capturing enclosing
# function locals, not this shape.


class Counter:
    def init(self):
        self.count = 0

    def render(self):
        def inc():
            self.count += 1

        inc()
        return self.count


c = Counter()
c.init()
assert c.render() == 1
assert c.count == 1
assert c.render() == 2
