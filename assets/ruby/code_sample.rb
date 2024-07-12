class Foo < Array
  include Enumerable
end

def bar(baz)
  @baz = baz
  $foo = Foo.new(1, 2, 3, 4)
end

baz = ->() {}
{
  "string" => bar(baz),
  hash_symbol: baz
}

print 1, 2.0

puts [0x3F, :symbol, '5']