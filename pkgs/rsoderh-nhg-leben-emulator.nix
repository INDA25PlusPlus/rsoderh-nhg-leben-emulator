{
  python313Packages,
}:

python313Packages.buildPythonApplication {
  pname = "rsoderh-nhg-leben-emulator";
  version = "0.1.0";
  pyproject = true;

  propagatedBuildInputs = with python313Packages; [
    textual
  ];

  build-system = with python313Packages; [ setuptools ];

  src = ../.;
}
