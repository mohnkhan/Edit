Name:           edit
Version:        0.1.0
Release:        1%{?dist}
Summary:        MS-DOS EDIT.COM reimplementation for Linux

License:        MIT
URL:            https://example.com/edit

BuildRequires:  rust >= 1.74.0
BuildRequires:  cargo

%description
Full-screen text editor for Linux terminals, faithfully reimplementing
the MS-DOS EDIT.COM interface with DOS-style blue-background UI,
pull-down menus, and F-key bindings.

%prep
# Source is unpacked by cargo build

%build
make release

%install
install -D -m 755 target/release/edit %{buildroot}%{_bindir}/edit
install -D -m 644 man/edit.1 %{buildroot}%{_mandir}/man1/edit.1

%files
%{_bindir}/edit
%{_mandir}/man1/edit.1

%changelog
* Thu Jun 18 2026 The edit contributors - 0.1.0-1
- Initial release
