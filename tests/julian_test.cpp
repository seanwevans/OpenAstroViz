#include "julian.hpp"
#include <cassert>
#include <cmath>

int main() {
    // Valid conversion: start of J2000
    auto jd1 = julian::julian_date_from_doy(2000, 1, 0.5);
    assert(jd1);
    assert(std::abs(*jd1 - 2451545.0) < 1e-6);

    // Another valid date
    auto jd2 = julian::julian_date_from_doy(2021, 275, 0.59097222);
    assert(jd2);
    assert(std::abs(*jd2 - 2459490.09097222) < 1e-6);

    // Out-of-range day-of-year for non-leap year should fail
    auto jd_invalid = julian::julian_date_from_doy(2021, 366, 0.0);
    assert(!jd_invalid);

    // Out-of-range day-of-year (zero) should also fail
    auto jd_zero = julian::julian_date_from_doy(2021, 0, 0.0);
    assert(!jd_zero);

    return 0;
}
