"""Small numeric helpers."""


def median(values):
    ordered = list(values)
    ordered.sort()
    middle = len(ordered) // 2
    if len(ordered) % 2 == 0:
        return (ordered[middle - 1] + ordered[middle]) / 2
    return ordered[len(ordered) // 2]


def clamp(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value
