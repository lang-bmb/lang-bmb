# ADR-0159: A package calls a package through the package manager

- Status: Accepted
- Date: 2026-11-26
- Related: ADR-0092 (the standard library), ADR-0155 decision 6 (one definition per path is not fixed without a reason), ADR-0156 (packages as working material), ROADMAP.md gate E

## Context

- The first official packages could call the standard library and nothing else. One of them copied about 50 lines of another (the specification and the functions of hexadecimal digits) because there was no way to call it.
- The compiler reads its files as one program. Top-level unit names are unique in a program, another unit is called by its full path, and the only root the compiler knows outside the program is `std`.
- A caller's proof sees only the contracts of what it calls, and the incremental cache does not prove an unchanged definition again.

## Decision

1. **The package manager passes the units of the dependencies to the compiler together with the caller's.** The caller writes the unit path (`hex::val(..)`). The compiler is not changed.
2. **Dependencies are paths.** A declaration names each dependency and the directory it is in. Dependencies of dependencies are followed; a cycle is an error. Fetching and choosing versions are not part of this.
3. **A naming rule keeps the flat space of unit names apart.** A unit of a package that is depended on has the package's name or begins with it and an underscore. The package manager checks it, and that unit names are distinct over the whole set. A package that is an executable is not depended on and is exempt.
4. **Proof covers the whole set.** A dependency's definitions are checked and proved with the caller's, and an escape hatch anywhere in the set fails the caller, as for the standard library. What a package declares and what its units call must agree.
5. **Compatibility is judged by contracts.** The version in a declaration is a label. Whether a new edition may replace an old one is decided by comparing contracts (`bmb weaken`), not by the label.

## Rationale

- No compiler change: the port's remaining parts and the measurement of gate D are the critical path, and a second root would be added to a compiler that is to retire and to its port.
- A second root (`pkg::NAME::unit::f`) would fix the package name into every path; ADR-0155 asks for a reason before fixing how names reach definitions, and there is none yet. Passing units fixes nothing new.
- The cost is a flat space of unit names, which the naming rule reduces to distinct package names inside one repository of packages.
- Reopened when one program needs two editions of one package at once; that is where naming by identity would be decided.
- Not measured: the cost of proving dependencies again; expected to be covered by the cache at the present sizes (seconds per package).
