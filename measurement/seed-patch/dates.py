"""Small calendar helpers."""


def is_leap(year):
    return year % 4 == 0


def quarter(month):
    return month // 3
