#!/usr/bin/env python3
"""
Extract method signatures from script crate's impl blocks
to generate proper trait definitions for Boa stubs.

Parses `impl XMethods<crate::DomTypeHolder> for Y { ... }` blocks
and extracts `fn` signatures to produce trait definitions.
"""

import os
import re
import sys
from collections import defaultdict

SCRIPT_DOM_DIR = os.path.join("components", "script", "dom")

def find_rust_files(base_dir):
    """Recursively find all .rs files."""
    for root, dirs, files in os.walk(base_dir):
        for f in files:
            if f.endswith('.rs'):
                yield os.path.join(root, f)

def extract_method_traits(filepath):
    """
    Extract trait impls from a Rust file.
    Returns dict: trait_name -> list of method signature strings
    """
    traits = defaultdict(list)
    
    with open(filepath, 'r', encoding='utf-8', errors='replace') as f:
        content = f.read()
    
    # Find all impl XMethods<...> for Y { ... } blocks
    # We need to match balanced braces
    pattern = r'impl\s+(\w+Methods)\s*<[^>]*>\s+for\s+\w+'
    
    for m in re.finditer(pattern, content):
        trait_name = m.group(1)
        start = m.start()
        
        # Find the opening brace
        brace_pos = content.find('{', m.end())
        if brace_pos == -1:
            continue
        
        # Match balanced braces
        depth = 1
        pos = brace_pos + 1
        while pos < len(content) and depth > 0:
            if content[pos] == '{':
                depth += 1
            elif content[pos] == '}':
                depth -= 1
            pos += 1
        
        block = content[brace_pos+1:pos-1]
        
        # Extract fn signatures from this block (top-level fns only)
        # We need only the signature, not the body
        fn_pattern = r'^\s*(fn\s+\w+\s*\([^)]*(?:\([^)]*\)[^)]*)*\)(?:\s*->\s*[^{]+)?)\s*\{'
        
        # Better approach: find each fn at depth 0
        lines = block.split('\n')
        in_fn = False
        brace_depth = 0
        current_sig = []
        
        for line in lines:
            stripped = line.strip()
            
            # Skip comments and doc comments
            if stripped.startswith('//') or stripped.startswith('///'):
                continue
            
            if not in_fn:
                # Look for fn keyword at top level (brace_depth == 0)
                if brace_depth == 0 and re.match(r'fn\s+\w+', stripped):
                    in_fn = True
                    current_sig = [stripped]
                    # Check if this line has the opening brace
                    for ch in stripped:
                        if ch == '{':
                            brace_depth += 1
                        elif ch == '}':
                            brace_depth -= 1
                    
                    if brace_depth > 0:
                        # Found opening brace, extract signature
                        sig = ' '.join(current_sig)
                        # Remove everything from the opening brace
                        sig = sig[:sig.rfind('{')].strip()
                        traits[trait_name].append(sig)
                        in_fn = False
                        # Continue counting braces
                        # Actually we need to skip the fn body
                        # brace_depth is already > 0
                else:
                    for ch in stripped:
                        if ch == '{':
                            brace_depth += 1
                        elif ch == '}':
                            brace_depth -= 1
            else:
                current_sig.append(stripped)
                for ch in stripped:
                    if ch == '{':
                        brace_depth += 1
                    elif ch == '}':
                        brace_depth -= 1
                
                if '{' in stripped:
                    sig = ' '.join(current_sig)
                    sig = sig[:sig.rfind('{')].strip()
                    traits[trait_name].append(sig)
                    in_fn = False
        
        # Wait for the fn body to end
        # Reset for next search
    
    return dict(traits)


def simplify_signature(sig):
    """
    Simplify a method signature for trait definition.
    Remove `crate::DomTypeHolder` references, etc.
    Just keep it as-is since we want exact match.
    """
    # Remove trailing whitespace/commas
    sig = sig.rstrip().rstrip(',')
    # Add semicolon for trait definition
    return sig + ";"


def main():
    all_traits = defaultdict(list)
    
    for filepath in find_rust_files(SCRIPT_DOM_DIR):
        traits = extract_method_traits(filepath)
        for name, methods in traits.items():
            # Deduplicate - same trait might be found in different files
            # but normally each trait is implemented once
            if name not in all_traits:
                all_traits[name] = methods
            else:
                # Merge methods
                existing = set(all_traits[name])
                for m in methods:
                    if m not in existing:
                        all_traits[name].append(m)
    
    # Print summary
    print(f"Found {len(all_traits)} trait implementations")
    total_methods = sum(len(m) for m in all_traits.values())
    print(f"Total methods: {total_methods}")
    print()
    
    # Print each trait
    for name in sorted(all_traits.keys()):
        methods = all_traits[name]
        print(f"// {name}: {len(methods)} methods")
        for method in methods:
            print(f"//   {method}")
        print()
    
    # Generate output
    output_path = os.path.join("components", "script_bindings", "trait_signatures.txt")
    with open(output_path, 'w', encoding='utf-8') as f:
        for name in sorted(all_traits.keys()):
            methods = all_traits[name]
            f.write(f"TRAIT:{name}:{len(methods)}\n")
            for method in methods:
                f.write(f"  {method}\n")
            f.write("\n")
    
    print(f"\nWritten to {output_path}")

if __name__ == '__main__':
    main()
