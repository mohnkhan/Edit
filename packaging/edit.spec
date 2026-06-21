Name:           edit
Version:        0.5.0
Release:        1%{?dist}
Summary:        MS-DOS EDIT.COM reimplementation for Linux

License:        MPL-2.0
URL:            https://github.com/mohnkhan/Edit

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
* Sun Jun 21 2026 The edit contributors - 0.5.0-1
- Release 0.5.0: rolls up features 038-050 (recent-files list, per-tab and
  per-pane soft-wrap, session scroll/selection/encoding restore, panic-surface
  hardening, and the app.rs module split). See CHANGELOG.md for full detail.
* Thu Jun 18 2026 The edit contributors - 0.1.0-1
- Initial release
