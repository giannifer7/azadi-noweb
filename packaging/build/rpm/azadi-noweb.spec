Name: azadi-noweb
Version: 0.1.3
Release: 1%{?dist}
Summary: A Rust implementation of noweb-style literate programming tool

License: MIT
URL: https://github.com/giannifer7/azadi-noweb
Source0: %{url}/archive/v%{version}.tar.gz

BuildRequires: cargo
Requires: glibc >= 2.28

%description
A Rust implementation of noweb-style literate programming tool

%prep
%autosetup

%build
cargo build --release

%install
install -Dm755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -Dm644 LICENSE %{buildroot}%{_datadir}/licenses/%{name}/LICENSE
install -Dm644 README.md %{buildroot}%{_datadir}/doc/%{name}/README.md

%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}