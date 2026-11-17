import pathlib,re,unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
class StandaloneCoordinateTests(unittest.TestCase):
 def test_direct_dependencies_are_exactly_pinned_git_not_sibling_paths(self):
  cargo=(ROOT/"Cargo.toml").read_text()
  self.assertNotRegex(cargo,r'path\s*=\s*"\.\./cliptown-')
  for name,revision in [('cliptown-client-rust','5c92f27f61be656408ffbc9822553387d2f648d4'),('cliptown-interfaces-rust','4782f0fbdaa2e4d4fdcbd637563c29d476f93f67')]:
   self.assertRegex(cargo,rf'(?m)^{name}\s*=\s*\{{[^\n]*rev = "{revision}"')
 def test_top_level_pins_do_not_count_as_complete_transitive_graph(self):
  status=(ROOT/"contracts/standalone-build-readiness.json").read_text()
  import json
  doc=json.loads(status)
  self.assertEqual(doc["promotion"],"blocked")
  self.assertFalse(doc["transitiveClientDependencyResolved"])
  self.assertFalse(doc["cleanCheckoutBuildExecuted"])
if __name__=="__main__":unittest.main()
