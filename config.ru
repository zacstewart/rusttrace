$LOAD_PATH.unshift File.join(File.dirname(__FILE__), 'lib')
require 'byebug'
require 'erb'
require 'ffi'
require 'rack'
require 'rusttrace/usage'

usage = Rusttrace::Usage.new
usage.attach

class Application
  TEMPLATE = ERB.new(File.read('lib/rusttrace/template.erb'))

  def initialize(usage)
    @usage = usage
  end

  def call(env)
    ['200', {'Content-Type' => 'text/html'}, [render_report]]
  end

  def render_report
    TEMPLATE.result(binding)
  end

  private

  attr_reader :usage

  def report
    usage.report
  end
end

app = Rack::Builder.new do
  map '/report' do
    run Application.new(usage)
  end

  map '/files' do
    run Rack::Directory.new('.')
  end

  map '/' do
    run lambda { |env|
      ['200', {'Content-Type' => 'text/html'}, ['recorded']]
    }
  end
end

run app.to_app
