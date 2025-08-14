#ifndef JULIAN_HPP
#define JULIAN_HPP

#include <cassert>
#include <cmath>

namespace julian {

inline bool is_leap_year(int year) {
    return (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
}

inline bool doy_to_month_day(int year, int doy, int &month, int &day) {
    const int max_doy = is_leap_year(year) ? 366 : 365;
    if (doy < 1 || doy > max_doy) {
        return false;
    }

    static const int days_in_month[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
    int m = 0;
    while (m < 12) {
        int dim = days_in_month[m];
        if (m == 1 && is_leap_year(year)) {
            dim = 29;
        }
        if (doy <= dim) {
            month = m + 1;
            day = doy;
            return true;
        }
        doy -= dim;
        ++m;
    }
    return false; // Should never reach here if input range is validated
}

inline double julian_date_from_calendar(int year, int month, int day, double frac_day) {
    int a = (14 - month) / 12;
    int y = year + 4800 - a;
    int m = month + 12 * a - 3;
    long jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;
    return jdn + frac_day - 0.5;
}

inline double julian_date_from_doy(int year, int doy, double frac_day) {
    int month, day;
    bool ok = doy_to_month_day(year, doy, month, day);
    assert(ok && "Day of year out of range");
    return julian_date_from_calendar(year, month, day, frac_day);
}

} // namespace julian

#endif // JULIAN_HPP
