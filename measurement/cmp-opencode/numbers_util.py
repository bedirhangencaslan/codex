"""Small numeric helpers."""


def median(values):
    ordered = list(values)
    return ordered[len(ordered) // 2]


def clamp(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value
