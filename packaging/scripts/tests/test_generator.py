import pytest
from pathlib import Path
import toml
from textwrap import dedent
from packaging.scripts.generate_packages import PackageGenerator, PackageMetadata

@pytest.fixture
def temp_project(tmp_path):
    """Create a temporary project structure."""
    project_dir = tmp_path / "test-project"

    # Create project structure
    (project_dir / "packaging/templates").mkdir(parents=True)
    (project_dir / "packaging/config").mkdir()
    (project_dir / "packaging/build").mkdir()

    # Create test Cargo.toml
    cargo_toml = project_dir / "Cargo.toml"
    cargo_toml.write_text(dedent("""
        [package]
        name = "test-package"
        version = "0.1.0"
        description = "A test package"
        license = "MIT"
        authors = ["Test Author <test@example.com>"]
        repository = "https://github.com/test/test-package"
    """))

    # Create test metadata.toml
    metadata_toml = project_dir / "packaging/config/metadata.toml"
    metadata_toml.write_text(dedent("""
        [build]
        output_dir = "build"

        [distributions]
        arch = ["x86_64", "aarch64"]

        [distributions.dependencies]
        alpine = ["musl"]
        void-musl = ["musl"]

        [distributions.libc]
        void-glibc = "glibc"
        void-musl = "musl"
    """))

    # Create test template
    template = project_dir / "packaging/templates/void.jinja2"
    template.write_text(dedent("""
        pkgname={{ package_name }}
        version={{ version }}
        {% if libc_variant == "musl" %}
        archs="~*-musl"
        {% endif %}
    """))

    return project_dir

def test_generator_initialization(temp_project):
    """Test PackageGenerator initialization."""
    generator = PackageGenerator(temp_project)
    assert generator.templates_dir.exists()
    assert generator.build_dir.exists()
    assert generator.config['distributions']['arch'] == ["x86_64", "aarch64"]

def test_metadata_from_cargo_toml(temp_project):
    """Test metadata extraction from Cargo.toml."""
    cargo_toml = temp_project / "Cargo.toml"
    generator = PackageGenerator(temp_project)
    metadata = PackageMetadata.from_cargo_toml(cargo_toml, generator.config)

    assert metadata.package_name == "test-package"
    assert metadata.version == "0.1.0"
    assert metadata.maintainer_name == "Test Author"
    assert metadata.maintainer_email == "test@example.com"

def test_void_musl_generation(temp_project):
    """Test generation of void-musl package."""
    generator = PackageGenerator(temp_project)
    metadata = PackageMetadata.from_cargo_toml(temp_project / "Cargo.toml", generator.config)

    generator.generate_format("void-musl", metadata)

    output_file = generator.build_dir / "void/musl/template"
    assert output_file.exists()
    content = output_file.read_text()
    assert 'archs="~*-musl"' in content

def test_clean_build_directory(temp_project):
    """Test cleaning build directory."""
    generator = PackageGenerator(temp_project)

    # Create some test files
    test_file = generator.build_dir / "test.txt"
    test_file.parent.mkdir(exist_ok=True)
    test_file.touch()

    generator.clean_build_directory()
    assert not generator.build_dir.exists()

def test_invalid_format(temp_project):
    """Test handling of invalid format."""
    generator = PackageGenerator(temp_project)
    metadata = PackageMetadata.from_cargo_toml(temp_project / "Cargo.toml", generator.config)

    with pytest.raises(Exception):
        generator.generate_format("invalid-format", metadata)

def test_missing_template(temp_project):
    """Test handling of missing template."""
    generator = PackageGenerator(temp_project)
    metadata = PackageMetadata.from_cargo_toml(temp_project / "Cargo.toml", generator.config)

    with pytest.raises(Exception):
        generator.generate_format("missing", metadata)
