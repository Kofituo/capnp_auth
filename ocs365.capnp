@0x9bf69a35c05a61ef;

struct Response
{
  description @0: Text;
  result @1: Text;
  message @2: Text;
  code @3: Int32;
}

interface Authenticate
{
  struct Request
  {
    userName @0: Text;
    userPassword @1: Text;
  }

  authenticate @0 (auth: Request) -> (result: Response);
}
