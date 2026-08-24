Name:           srsq-panel
Version:        0.3.2
Release:        1
Summary:        Custom modular Wayland status panel for Sway
License:        MIT
URL:            https://github.com/Hitoshi-hub/srsq-panel

%description
Custom modular Wayland status panel for Sway

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/srsq-panel
install -m 0755 /home/hitoshi/tmp/srsq-panel/target/release/srsq-panel %{buildroot}%{_bindir}/srsq-panel
if [ -d "resources" ]; then
    cp -r resources/* %{buildroot}%{_datadir}/srsq-panel/
fi

%files
%{_bindir}/srsq-panel
%{_datadir}/srsq-panel