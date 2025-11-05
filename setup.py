from setuptools import setup, find_packages

name = 'rsoderh-nhg-leben-emulator'

setup(
    name=name,
    version='0.1.0',
    # Modules to import from other scripts:
    packages=find_packages(),
    # Executables
    py_modules=["src.main"],
    entry_points={
        'console_scripts': [f'{name}=src.main:main']
    }
)