Name:           rust-rlg
Version:        0.0.7
Release:        1%{?dist}
Summary:        High-performance lock-free observability engine for Rust

License:        MIT or Apache-2.0
URL:            https://github.com/sebastienrousseau/rlg
Source0:        %{url}/archive/v%{version}.tar.gz

BuildRequires:  rust-toolset >= 1.87.0
BuildRequires:  systemd-devel
BuildRequires:  gcc

%description
RLG (RustLogs) is a brutalist, zero-allocation observability engine 
designed for the 2026 industry standards. It features a lock-free 
LMAX Disruptor ingestion engine and native platform sinks.

%prep
%autosetup -n rlg-%{version}

%build
cargo build --release --all-features

%check
cargo test --release --all-features

%install
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/rlg %{buildroot}%{_bindir}/rlg
mkdir -p %{buildroot}%{_licensedir}/%{name}
install -m 0644 LICENSE-MIT %{buildroot}%{_licensedir}/%{name}/LICENSE-MIT
install -m 0644 LICENSE-APACHE %{buildroot}%{_licensedir}/%{name}/LICENSE-APACHE

%files
%{_bindir}/rlg
%license LICENSE-MIT LICENSE-APACHE
%doc README.md

%changelog
* Thu Mar 05 2026 Sebastien Rousseau <sebastienrousseau@users.noreply.github.com> - 0.0.7-1
- Initial v0.0.7 release with Lock-Free Engine and AI-native formats.
