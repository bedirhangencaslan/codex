"""Small numeric helpers."""


def median(values):
    ordered = sorted(values)
    n = len(ordered)
    mid = n // 2
    if n % 2 == 1:
        return ordered[mid]
    return (ordered[mid - 1] + ordered[mid]) / 2


def clamp(value, low, high):
    if value < low:
        return low
    if value > high:
        return high
    return value
