use std::prelude::v1::*;
pub fn get(name: &str) -> &'static str {
    match name {
        "simple_local" => r#"
{
  "Block": [
    {
      "Local": [
        [
          {
            "Id": "a"
          }
        ],
        [
          {
            "Add": [
              {
                "Number": 1
              },
              {
                "Mul": [
                  {
                    "Number": 2
                  },
                  {
                    "Number": 3
                  }
                ]
              }
            ]
          }
        ]
      ]
    }
  ]
}
        "#,
        "function_def_call" => r#"
{
  "Block": [
    {
      "Set": [
        [
          {
            "Id": "f"
          }
        ],
        [
          {
            "Function": [
              [
                {
                  "Id": "a"
                }
              ],
              {
                "Block": [
                  {
                    "Return": [
                      {
                        "Add": [
                          {
                            "Id": "a"
                          },
                          {
                            "Number": 1
                          }
                        ]
                      }
                    ]
                  }
                ]
              }
            ]
          }
        ]
      ]
    },
    {
      "Local": [
        [
          {
            "Id": "v"
          }
        ],
        [
          {
            "Number": 1
          }
        ]
      ]
    },
    {
      "Call": [
        {
          "Id": "print"
        },
        [
          {
            "Call": [
              {
                "Id": "f"
              },
              [
                {
                  "Id": "v"
                }
              ]
            ]
          }
        ]
      ]
    }
  ]
}
        "#,
        "loops" => r#"
{
  "Block": [
    {
      "Fornum": [
        {
          "Id": "i"
        },
        {
          "Number": 1
        },
        {
          "Number": 10
        },
        null,
        {
          "Block": [
            {
              "Local": [
                [
                  {
                    "Id": "a"
                  }
                ],
                [
                  {
                    "Id": "i"
                  }
                ]
              ]
            }
          ]
        }
      ]
    },
    {
      "Fornum": [
        {
          "Id": "i"
        },
        {
          "Number": 1
        },
        {
          "Number": 10
        },
        {
          "Number": 1
        },
        {
          "Block": [
            {
              "Local": [
                [
                  {
                    "Id": "a"
                  }
                ],
                [
                  {
                    "Id": "i"
                  }
                ]
              ]
            }
          ]
        }
      ]
    },
    {
      "Local": [
        [
          {
            "Id": "a"
          }
        ],
        [
          {
            "Table": [
              {
                "Number": 1
              },
              {
                "Number": 2
              }
            ]
          }
        ]
      ]
    },
    {
      "Forin": [
        [
          {
            "Id": "i"
          },
          {
            "Id": "v"
          }
        ],
        [
          {
            "Call": [
              {
                "Id": "ipairs"
              },
              [
                {
                  "Id": "a"
                }
              ]
            ]
          }
        ],
        {
          "Block": [
            {
              "Local": [
                [
                  {
                    "Id": "t"
                  }
                ],
                [
                  {
                    "Id": "v"
                  }
                ]
              ]
            }
          ]
        }
      ]
    },
    {
      "Local": [
        [
          {
            "Id": "i"
          }
        ],
        [
          {
            "Number": 0
          }
        ]
      ]
    },
    {
      "While": [
        {
          "Lt": [
            {
              "Id": "i"
            },
            {
              "Number": 10
            }
          ]
        },
        {
          "Block": [
            {
              "Set": [
                [
                  {
                    "Id": "i"
                  }
                ],
                [
                  {
                    "Add": [
                      {
                        "Id": "i"
                      },
                      {
                        "Number": 1
                      }
                    ]
                  }
                ]
              ]
            }
          ]
        }
      ]
    },
    {
      "Repeat": [
        {
          "Block": [
            {
              "Set": [
                [
                  {
                    "Id": "i"
                  }
                ],
                [
                  {
                    "Sub": [
                      {
                        "Id": "i"
                      },
                      {
                        "Number": 1
                      }
                    ]
                  }
                ]
              ]
            }
          ]
        },
        {
          "Eq": [
            {
              "Id": "i"
            },
            {
              "Number": 0
            }
          ]
        }
      ]
    }
  ]
}
        "#,
        "arithmetic" => r#"
{
  "Block": [
    {
      "Local": [
        [
          {
            "Id": "a"
          }
        ],
        [
          {
            "Add": [
              {
                "Number": 1
              },
              {
                "Mul": [
                  {
                    "Number": 2
                  },
                  {
                    "Number": 3
                  }
                ]
              }
            ]
          }
        ]
      ]
    },
    {
      "Local": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Boolean": false
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Lt": [
              {
                "Id": "a"
              },
              {
                "Number": 7
              }
            ]
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Le": [
              {
                "Id": "a"
              },
              {
                "Number": 7
              }
            ]
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Eq": [
              {
                "Id": "a"
              },
              {
                "Number": 7
              }
            ]
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Not": {
              "Eq": [
                {
                  "Id": "a"
                },
                {
                  "Number": 7
                }
              ]
            }
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Le": [
              {
                "Number": 7
              },
              {
                "Id": "a"
              }
            ]
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Lt": [
              {
                "Number": 7
              },
              {
                "Id": "a"
              }
            ]
          }
        ]
      ]
    },
    {
      "Set": [
        [
          {
            "Id": "b"
          }
        ],
        [
          {
            "Not": {
              "Boolean": false
            }
          }
        ]
      ]
    }
  ]
}
        "#,
        _ => unimplemented!()
    }
}
