"""Focused safety/integrity tests for the private-archive restore tool."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

from restore import restore, target_path


class RestoreTests(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.base=Path(self.temp.name)
        self.roots={'repo':self.base/'repo','audit':self.base/'audit'}
        self.payload=b'private research witness\x00\xff'
        self.digest=hashlib.sha256(self.payload).hexdigest()
        archive=self.base/'part.zip'
        with zipfile.ZipFile(archive,'w') as z:
            z.writestr('objects/'+self.digest,self.payload)
        self.manifest=self.base/'manifest.json'
        self.data={'schema':1,'assets':[{'name':'part.zip','bytes':archive.stat().st_size,
                    'sha256':hashlib.sha256(archive.read_bytes()).hexdigest()}],
                   'files':[{'asset':'part.zip','member':'objects/'+self.digest,
                             'sha256':self.digest,'bytes':len(self.payload),
                             'targets':['repo/test-output/a.bin','audit/fixtures/b.bin']}]}
        self.save()

    def tearDown(self):self.temp.cleanup()
    def save(self):self.manifest.write_text(json.dumps(self.data))
    def run_restore(self,verify=False):
        with contextlib.redirect_stdout(io.StringIO()):
            return restore(self.base,self.manifest,self.roots,verify)

    def test_two_roots_and_identical_resume(self):
        result=self.run_restore()
        self.assertEqual(result['paths_restored'],2)
        self.assertEqual((self.roots['repo']/'test-output/a.bin').read_bytes(),self.payload)
        self.assertEqual(self.run_restore()['existing_identical_paths'],2)

    def test_verify_only_writes_nothing(self):
        self.assertEqual(self.run_restore(True)['unique_files_verified'],1)
        self.assertFalse(self.roots['repo'].exists())

    def test_collision_refused_before_writes(self):
        p=self.roots['audit']/'fixtures/b.bin';p.parent.mkdir(parents=True);p.write_bytes(b'existing work')
        with self.assertRaises(ValueError):self.run_restore()
        self.assertEqual(p.read_bytes(),b'existing work')
        self.assertFalse(self.roots['repo'].exists())

    def test_archive_corruption_refused(self):
        with (self.base/'part.zip').open('ab') as stream:stream.write(b'bad')
        with self.assertRaises(ValueError):self.run_restore()
        self.assertFalse(self.roots['repo'].exists())

    def test_traversal_and_absolute_paths_refused(self):
        for name in ['repo/../outside.bin','repo/C:/outside','/repo/absolute','repo/a\\b']:
            with self.subTest(name=name),self.assertRaises(ValueError):
                target_path(name,self.roots)

    def test_duplicate_destination_refused(self):
        self.data['files'][0]['targets'].append('repo/test-output/a.bin');self.save()
        with self.assertRaises(ValueError):self.run_restore()

    def test_wrong_member_digest_refused(self):
        self.data['files'][0]['bytes']+=1;self.save()
        with self.assertRaises(ValueError):self.run_restore()
        self.assertFalse(self.roots['repo'].exists())


if __name__=='__main__':unittest.main()
