This project is a WIP. This README will be updated with more information over time.

In short, this is a safe wrapper around Windows Critical Sections.

See tests in crit.rs and crit_static.rs for usage examples.

This crate currently has no notion of a "poisoned" critical section, although it does not currently require such a notion for safety. This will need to change prior to introducing a Critical Section based Mutex type.

See Safety.md for a list of safety considerations around the imlementation of this crate.

Feedback and contributions are always welcome.