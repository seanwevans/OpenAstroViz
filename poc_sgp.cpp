// poc_sgp4.cpp — minimal proof‑of‑concept for OpenAstroViz
// MIT License © 2025 OpenAstroViz Contributors
// -----------------------------------------------------------
// This is NOT a production‑grade SGP4.  It is a stripped‑down
// demonstrator that:
//   • parses a Two‑Line‑Element set (TLE)
//   • converts it into classical orbital elements (Keplerian)
//   • propagates via simple two‑body Kepler drift (f & g series)
//   • prints ECI XYZ every Δt seconds for 1 orbit
//
// Replace with a full Vallado SGP4 implementation in `core/`.
// -----------------------------------------------------------
#include <cmath>
#include <cstdlib>
#include <iomanip>
#include <iostream>
#include <string>

#include "julian.hpp"

constexpr double kPi = 3.14159265358979323846;
constexpr double kEarthMu = 3.986004418e14; // m^3 s^‑2
constexpr double kEarthRad = 6378.137e3;    // m (equatorial)
constexpr double deg2rad(double d) { return d * kPi / 180.0; }

struct Tle {
    std::string line1;
    std::string line2;
};

struct Orbit {
    double epoch_jd;    // Julian Date
    double inc;         // inclination           [rad]
    double raan;        // right asc. node       [rad]
    double ecc;         // eccentricity          [unitless]
    double argp;        // argument of perigee   [rad]
    double mean_anom;   // mean anomaly          [rad]
    double mean_motion; // rev / day             [rad/s after conv.]
};

// --- very naïve checksum ignore ---
static Orbit parse_tle(const Tle &tle) {
    Orbit o{};
    // Epoch (yyddd.dddddd)
    int yy = std::stoi(tle.line1.substr(18, 2));
    int doy = std::stoi(tle.line1.substr(20, 3));
    double frac_day = std::stod(tle.line1.substr(23, 8));
    int year = (yy < 57 ? 2000 + yy : 1900 + yy);
    o.epoch_jd = julian::julian_date_from_doy(year, doy, frac_day);

    o.inc = deg2rad(std::stod(tle.line2.substr(8, 8)));
    o.raan = deg2rad(std::stod(tle.line2.substr(17, 8)));
    o.ecc = std::stod("0." + tle.line2.substr(26, 7));
    o.argp = deg2rad(std::stod(tle.line2.substr(34, 8)));
    o.mean_anom = deg2rad(std::stod(tle.line2.substr(43, 8)));
    double mm_rev_day = std::stod(tle.line2.substr(52, 11));
    o.mean_motion = mm_rev_day * 2.0 * kPi / 86400.0; // rad/s
    return o;
}

// Kepler solver (Newton‑Raphson on eccentric anomaly)
static double solve_kepler(double M, double e, int iters = 10) {
    double E = M;
    for (int i = 0; i < iters; ++i) {
        double f = E - e * sin(E) - M;
        double f_p = 1.0 - e * cos(E);
        E -= f / f_p;
    }
    return E;
}

// Propagate Δt seconds via Kepler two‑body
static void propagate(const Orbit &o, double dt, double &x, double &y, double &z) {
    double a = pow(kEarthMu / (o.mean_motion * o.mean_motion), 1.0 / 3.0); // semi‑major
    double n = o.mean_motion;
    double M = o.mean_anom + n * dt;
    double E = solve_kepler(fmod(M, 2.0 * kPi), o.ecc);
    double v = 2.0 * atan2(sqrt(1 + o.ecc) * sin(E / 2), sqrt(1 - o.ecc) * cos(E / 2));
    double r = a * (1 - o.ecc * cos(E));

    // Perifocal coordinates
    double xp = r * cos(v);
    double yp = r * sin(v);

    // Rotation to ECI
    double cos_raan = cos(o.raan), sin_raan = sin(o.raan);
    double cos_inc = cos(o.inc), sin_inc = sin(o.inc);
    double cos_argp = cos(o.argp), sin_argp = sin(o.argp);

    double x_eci = (cos_raan * cos_argp - sin_raan * sin_argp * cos_inc) * xp +
                   (-cos_raan * sin_argp - sin_raan * cos_argp * cos_inc) * yp;
    double y_eci = (sin_raan * cos_argp + cos_raan * sin_argp * cos_inc) * xp +
                   (-sin_raan * sin_argp + cos_raan * cos_argp * cos_inc) * yp;
    double z_eci = (sin_argp * sin_inc) * xp + (cos_argp * sin_inc) * yp;

    x = x_eci;
    y = y_eci;
    z = z_eci;
}

int main() {
    std::cout << "# Minimal OpenAstroViz C++ PoC\n";
    std::cout << "Enter two TLE lines separated by newlines:\n";
    Tle tle;
    std::getline(std::cin, tle.line1);
    std::getline(std::cin, tle.line2);
    if (tle.line1.empty() || tle.line2.empty()) {
        std::cerr << "TLE input error.\n";
        return EXIT_FAILURE;
    }

    Orbit orb = parse_tle(tle);
    double period = 2.0 * kPi / orb.mean_motion;
    double dt_step = 60.0; // seconds
    int steps = static_cast<int>(period / dt_step);

    std::cout << std::fixed << std::setprecision(2);
    for (int i = 0; i <= steps; ++i) {
        double dt = i * dt_step;
        double x, y, z;
        propagate(orb, dt, x, y, z);
        std::cout << std::setw(8) << dt << " s : " << std::setprecision(1) << std::setw(10)
                  << x / 1000.0 << " km  " << std::setw(10) << y / 1000.0 << " km  "
                  << std::setw(10) << z / 1000.0 << " km" << std::setprecision(2) << "\n";
    }
    return EXIT_SUCCESS;
}

/*
Compile (C++17):
    g++ -O2 -std=c++17 poc_sgp4.cpp -o poc_sgp4

Run:
    ./poc_sgp4
    <paste two TLE lines>

Limitations:
    • Uses Keplerian two‑body, not SGP4 perturbations → ~km error per day.
    • Ignores B* drag, J2, J3, etc.
    • Good enough to validate IO, epoch math, ECI transforms.
*/
