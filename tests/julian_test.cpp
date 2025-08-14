#include <catch2/catch_test_macros.hpp>
#include <cmath>

#include "julian.hpp"

int main() {
    double jd1 = julian::julian_date_from_doy(2000, 1, 0.5);
    assert(std::abs(jd1 - 2451545.0) < 1e-6);

    double jd2 = julian::julian_date_from_doy(2021, 275, 0.59097222);
    assert(std::abs(jd2 - 2459490.09097222) < 1e-6);
    return 0;
}
