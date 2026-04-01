#include "julian.hpp"

#include <cassert>
#include <cmath>
#include <cstdlib>
#include <iostream>

namespace {

void test_julian_date_from_doy() {
    auto jd1 = julian::julian_date_from_doy(2000, 1, 0.5);
    assert(jd1.has_value());
    assert(std::abs(*jd1 - 2451545.0) < 1e-6);

    auto jd2 = julian::julian_date_from_doy(2021, 275, 0.59097222);
    assert(jd2.has_value());
    assert(std::abs(*jd2 - 2459490.09097222) < 1e-6);
}

void test_doy_to_month_day_valid_and_invalid_inputs() {
    int month = 0;
    int day = 0;

    bool ok = julian::doy_to_month_day(2021, 275, month, day);
    assert(ok);
    assert(month == 10);
    assert(day == 2);

    ok = julian::doy_to_month_day(2020, 366, month, day);
    assert(ok);
    assert(month == 12);
    assert(day == 31);

    ok = julian::doy_to_month_day(2021, 366, month, day);
    assert(!ok);
}

} // namespace

int main() {
    test_julian_date_from_doy();
    test_doy_to_month_day_valid_and_invalid_inputs();

    std::cout << "All C++ tests passed.\n";
    return EXIT_SUCCESS;
}
