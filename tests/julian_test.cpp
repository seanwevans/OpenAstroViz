#include "julian.hpp"
#include <cassert>
#include <cmath>

int main() {
    double jd1 = julian_date_from_doy(2000, 1, 0.5);
    assert(std::abs(jd1 - 2451545.0) < 1e-6);

    double jd2 = julian_date_from_doy(2021, 275, 0.59097222);
    assert(std::abs(jd2 - 2459490.09097222) < 1e-6);

    int month, day;
    bool ok = doy_to_month_day(2021, 366, month, day);
    assert(!ok);
    return 0;
}
