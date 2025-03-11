import random
import re

class SpinTax:
    @staticmethod
    def parse(text):
        """Parse and process SpinTax syntax to generate a random variation"""
        while '{' in text:
            text = re.sub(
                r'{([^{}]*)}',
                lambda m: random.choice(m.group(1).split('|')).strip(),
                text
            )
        return text

    @staticmethod
    def get_variations_count(text):
        """Calculate total possible variations of a SpinTax message"""
        count = 1
        while '{' in text:
            options = re.search(r'{([^{}]*)}', text).group(1).split('|')
            count *= len(options)
            text = re.sub(r'{([^{}]*)}', '', text, 1)
        return count 