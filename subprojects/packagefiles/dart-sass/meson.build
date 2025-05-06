project('dart-sass')

if meson.is_subproject()
  if host_machine.system() == 'windows'
    meson.override_find_program('sass-win32-x86_64', files('sass.bat'))
  else
    # Assume linux
    meson.override_find_program('sass-linux-x86_64', files('sass'))
  endif
endif
